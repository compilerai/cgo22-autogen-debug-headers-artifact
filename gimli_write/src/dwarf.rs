use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashMap;

use gimli::constants::{
    DW_AT_const_value, DW_AT_high_pc, DW_AT_location, DW_AT_low_pc, DW_OP_div, DW_OP_minus,
    DW_OP_mod, DW_OP_mul, DW_OP_plus, DW_OP_stack_value,
};
use gimli::read::EndianSlice;
use gimli::read::Reader;
use gimli::write::{
    Address, Attribute, AttributeValue, DebuggingInformationEntry, EndianVec, StringTable, Unit,
    UnitEntryId,
};
use gimli::{self, read, write, LittleEndian};
use object::write as object_write;
use object::{self, Object, ObjectSection, SymbolIndex};
use std::convert::TryInto;
use std::io::Read;
use std::str;

/* See if using write::Address::Constant(addr) is correct and if we can use this for relocatable
 * addresses! */
/* See if low_pc + high_pc or just high_pc, wherever it is used, whether it is correct or not */

#[derive(Clone)]
struct Location<R: Reader> {
    expression: read::Expression<R>,
    begin: u64,
    end: u64,
}
#[derive(Clone)]
struct WriteLocation {
    expression: write::Expression,
    begin: u64,
    end: u64,
}
#[derive(Clone)]
enum LocationInfo<R: Reader> {
    LocList(Vec<read::LocationListEntry<R>>),
    Loc(Location<R>),
    WLoc(WriteLocation),
    IntLocList(Vec<write::Location>), /*Intermediate Location List*/
}

fn get_die<'a>(unit: &'a mut Unit, entity_id: &UnitEntryId) -> &'a mut DebuggingInformationEntry {
    let entity_die = unit.get_mut(*entity_id);
    entity_die
}

fn get_var_loc<'a>(unit: &'a mut Unit, var_id: &UnitEntryId) -> Option<&'a mut Attribute> {
    let var_die = unit.get_mut(*var_id);
    let mut var_location = None;
    for attr in var_die.attrs_mut() {
        let attr_type = attr.name().static_string().unwrap();
        if attr_type == "DW_AT_location" {
            var_location = Some(attr);
            break;
        }
    }
    var_location
}

fn get_var(
    unit: &Unit,
    func_id: &UnitEntryId,
    strings: &StringTable,
    var_name: &str,
    expr_rng: (u64, u64),
    no_of_vars: u8,
) -> (Option<UnitEntryId>, Option<UnitEntryId>) {
    let depth = 0;
    let mut low_pc = None;
    let mut high_pc = None;
    {
        let func_die = unit.get(*func_id);
        let attr_val = func_die.get(DW_AT_low_pc);
        if let Some(write::AttributeValue::Address(addr)) = attr_val {
            let addr = get_addr(*addr);
            low_pc = Some(addr);
        } else {
            eprintln!("function's low_pc attribute unexpected! -- not handled!");
        }
        let attr_val = func_die.get(DW_AT_high_pc);
        if let Some(write::AttributeValue::Udata(offset)) = attr_val {
            high_pc = Some(*offset);
        } else {
            eprintln!("function's high_pc attribute unexpected! -- not handled!");
        }
    }
    let (var_id, parent_id, _depth) = get_var_depth(
        unit,
        func_id,
        strings,
        var_name,
        depth,
        expr_rng,
        (low_pc, high_pc),
        no_of_vars,
    );
    (var_id, parent_id)
}

fn get_var_depth(
    unit: &Unit,
    func_id: &UnitEntryId,
    strings: &StringTable,
    var_name: &str,
    depth: i64,
    expr_rng: (u64, u64),
    curr_rng: (Option<u64>, Option<u64>),
    no_of_vars: u8,
) -> (Option<UnitEntryId>, Option<UnitEntryId>, i64) {
    let lex_blk_die = unit.get(*func_id);
    let mut var_id_depth = (None, None, -1);
    for child in lex_blk_die.children() {
        let child_die = unit.get(*child);

        let mut low_pc = curr_rng.0;
        let mut high_pc = curr_rng.1;

        let attr_val = child_die.get(DW_AT_low_pc);
        if let Some(write::AttributeValue::Address(addr)) = attr_val {
            let addr = get_addr(*addr);
            low_pc = Some(addr);
            println!("got a new low_pc : {}", addr);
        } else {
            println!("low_pc: Unexpected AttributeValue type! {:?}", attr_val);
        }
        let attr_val = child_die.get(DW_AT_high_pc);
        if let Some(write::AttributeValue::Udata(offset)) = attr_val {
            high_pc = Some(*offset);
            println!("got a new high_pc : {}", offset);
        } else {
            println!("high_pc: Unexpected AttributeValue type! {:?}", attr_val);
        }

        let s = child_die.tag().static_string().unwrap();
        if s == "DW_TAG_variable" || s == "DW_TAG_formal_parameter" {
            let attr_val = child_die.get(gimli::constants::DW_AT_name).unwrap();
            println!("attr_val = {:?}", attr_val);
            if let AttributeValue::StringRef(string_id) = attr_val {
                println!(
                    "string {} {}",
                    str::from_utf8(strings.get(*string_id)).unwrap(),
                    var_name
                );
                //let tmpstr: &str = str::from_utf8(strings.get(*string_id)).unwrap();
                //println!("length {} {}", String::from(tmpstr).len(), String::from(var_name.trim()).len());
                if str::from_utf8(strings.get(*string_id)).unwrap() == var_name.trim() {
                    println!("matched variable name -- strings section");
                    if no_of_vars == 1 {
                        if curr_rng.0 != None && curr_rng.1 != None {
                            if !(curr_rng.0.unwrap() >= expr_rng.1
                                || (curr_rng.0.unwrap() + curr_rng.1.unwrap()) <= expr_rng.0)
                            {
                                eprintln!("curr_rng_low: {}, curr_rng_low+high: {}, expr_rng_low: {}, expr_rng_high: {}", curr_rng.0.unwrap(), curr_rng.0.unwrap() + curr_rng.1.unwrap(), expr_rng.0, expr_rng.1);
                                var_id_depth = (Some(*child), Some(*func_id), depth + 1);
                                return var_id_depth;
                            }
                        }
                    } else {
                        if curr_rng.0 != None && curr_rng.1 != None {
                            if curr_rng.0.unwrap() <= expr_rng.0
                                && (curr_rng.0.unwrap() + curr_rng.1.unwrap()) >= expr_rng.1
                            {
                                if var_id_depth.2 == -1 || depth + 1 > var_id_depth.2 {
                                    var_id_depth = (Some(*child), Some(*func_id), depth + 1);
                                } else {
                                    println!("depth outside!");
                                }
                            } else {
                                println!("range outside!");
                            }
                        } else {
                            /*if var_id_depth.1 == -1 || depth + 1 > var_id_depth.1 {
                                var_id_depth = (Some(*child), depth + 1);
                            }
                            else {
                                println!("depth outside!");
                            }*/
                        }
                    }
                }
            /*else {
                println!("not matched");
            }*/
            } else if let AttributeValue::String(vec_bytes) = attr_val {
                if String::from_utf8(vec_bytes.to_vec()).unwrap() == var_name.trim() {
                    /* same code block as present in above if StringRef clause */
                    println!("matched variable name -- direct string");
                    if no_of_vars == 1 {
                        if curr_rng.0 != None && curr_rng.1 != None {
                            if !(curr_rng.0.unwrap() >= expr_rng.1
                                || (curr_rng.0.unwrap() + curr_rng.1.unwrap()) <= expr_rng.0)
                            {
                                eprintln!("curr_rng_low: {}, curr_rng_low+high: {}, expr_rng_low: {}, expr_rng_high: {}", curr_rng.0.unwrap(), curr_rng.0.unwrap() + curr_rng.1.unwrap(), expr_rng.0, expr_rng.1);
                                var_id_depth = (Some(*child), Some(*func_id), depth + 1);
                                return var_id_depth;
                            }
                        }
                    } else {
                        if curr_rng.0 != None && curr_rng.1 != None {
                            if curr_rng.0.unwrap() <= expr_rng.0
                                && (curr_rng.0.unwrap() + curr_rng.1.unwrap()) >= expr_rng.1
                            {
                                if var_id_depth.2 == -1 || depth + 1 > var_id_depth.2 {
                                    var_id_depth = (Some(*child), Some(*func_id), depth + 1);
                                } else {
                                    println!("depth outside!");
                                }
                            } else {
                                println!(
                                    "range outside! curr_rng: {:?} expr_rng: {:?}",
                                    curr_rng, expr_rng
                                );
                            }
                        } else {
                            /*if var_id_depth.1 == -1 || depth + 1 > var_id_depth.1 {
                                var_id_depth = (Some(*child), depth + 1);
                            }
                            else {
                                println!("depth outside!");
                            }*/
                        }
                    }
                }
            } else {
                println!("Attribute Value of this type not handled yet!\n");
            }
        }

        let (var_id_rec, parent_id_rec, depth_rec) = get_var_depth(
            unit,
            child,
            strings,
            var_name,
            depth + 1,
            expr_rng,
            (low_pc, high_pc),
            no_of_vars,
        );
        if var_id_rec != None {
            /* no need of the below if block -- but, using it just to be explicit */
            if no_of_vars == 1 {
                return (var_id_rec, parent_id_rec, depth_rec);
            }
            if var_id_depth.2 == -1 || depth_rec > var_id_depth.2 {
                var_id_depth = (var_id_rec, parent_id_rec, depth_rec)
            }
        }
    }
    var_id_depth
}

fn get_func_id(
    unit: &Unit,
    strings: &StringTable,
    function: &str,
) -> Option<gimli::write::UnitEntryId> {
    let root_id = unit.root();
    let root_die = unit.get(root_id);
    let mut var_id = None;
    for child in root_die.children() {
        let child_die = unit.get(*child);
        let s = child_die.tag().static_string().unwrap();
        if s == "DW_TAG_subprogram" {
            for attr in child_die.attrs() {
                let attr_type = attr.name().static_string().unwrap();
                let attr_val = attr.get();
                if attr_type == "DW_AT_name" {
                    if let AttributeValue::StringRef(string_id) = attr_val {
                        if str::from_utf8(strings.get(*string_id)).unwrap() == function.trim() {
                            var_id = Some(*child);
                        }
                    } else if let AttributeValue::String(vec_bytes) = attr_val {
                        if String::from_utf8(vec_bytes.to_vec()).unwrap() == function.trim() {
                            var_id = Some(*child);
                        }
                    } else {
                        println!("Attribute Value of this type not handled yet!\n");
                    }
                }
            }
        }
    }
    var_id
}

fn get_register_mapping(reg_name: &str) -> u16 {
    match reg_name {
        "eax" => 0,
        "ecx" => 1,
        "edx" => 2,
        "ebx" => 3,
        "esp" => 4,
        "ebp" => 5,
        "esi" => 6,
        "edi" => 7,
        //"xmm0"|"xmm1"|"xmm2"|"xmm3"|"xmm4"|"xmm5"|"xmm6"|"xmm7" => 21,
        //_ => 255,
        "es" => 40,
        "cs" => 41,
        "ss" => 42,
        "ds" => 43,
        "fs" => 44,
        "gs" => 45,
        _ => {
            if reg_name.starts_with("xmm") {
                let num: u16 = reg_name[3..].parse().unwrap();
                21 + num
            } else {
                255
            }
        }
    }
}

fn is_arith_op(s: &str) -> bool {
    match s {
        "+" | "*" | "/" | "-" | "%" => true,
        _ => false,
    }
}

fn get_arith_dwop(s: &str) -> Option<gimli::constants::DwOp> {
    match s {
        "+" => Some(DW_OP_plus),
        "-" => Some(DW_OP_minus),
        "*" => Some(DW_OP_mul),
        "/" => Some(DW_OP_div),
        "%" => Some(DW_OP_mod),
        _ => None,
    }
}

fn create_dwarf_expr(loc_expr: &str) -> write::Expression {
    let loc_expr_vec: Vec<&str> = loc_expr.split_ascii_whitespace().collect();
    let mut new_expr = gimli::write::Expression::new();

    for component in loc_expr_vec {
        if component.chars().nth(0).unwrap() == '%' {
            //register
            /*for regnum in 0..255 {
                //new_expr.op(DW_OP_mul);
                new_expr.op_breg(gimli::Register(regnum), 0);
            }*/
            let reg = get_register_mapping(&component[1..]);
            new_expr.op_breg(gimli::Register(reg), 0);
        } else if let Ok(number) = component.parse::<i64>() {
            new_expr.op_consts(number);
        } else if is_arith_op(component) {
            new_expr.op(get_arith_dwop(component).unwrap());
        } else {
            println!("Invalid component string in location expression!");
        }
    }
    new_expr.op(DW_OP_stack_value);
    /*new_expr.op_breg(gimli::Register(0), 0);
    new_expr.op_consts(128000);
    new_expr.op(DW_OP_plus);
    new_expr.op(DW_OP_lit4);
    new_expr.op(DW_OP_div);
    new_expr.op(DW_OP_stack_value);*/
    new_expr
}

fn get_func_entry_offset<T: Reader>(
    dwarf: &gimli::read::Dwarf<T>,
    func_name: &str,
) -> (
    Option<gimli::read::UnitOffset<T::Offset>>,
    Option<u64>,
    Option<u64>,
) {
    let units = &mut dwarf.units();
    let unit_header = units.next().unwrap();
    let unit = dwarf.unit(unit_header.unwrap()).unwrap();
    let mut entries = unit.entries();
    let mut func_entry_offset = None;
    let mut func_start_addr = None;
    let mut func_end_offset = None;
    loop {
        if let Some((_, entry)) = entries.next_dfs().unwrap() {
            if entry.tag() == gimli::DW_TAG_subprogram {
                let mut attrs = entry.attrs();
                let mut flag = false;
                while let Some(attr) = attrs.next().unwrap() {
                    if attr.name().static_string().unwrap() == "DW_AT_name" {
                        if let read::AttributeValue::DebugStrRef(debug_str_offset) = attr.value() {
                            let str_val = dwarf.string(debug_str_offset).unwrap();
                            if str_val.to_string().unwrap() == func_name {
                                eprintln!("function {} FOUND!", func_name);
                                flag = true;
                                //break;
                            }
                        } else if let read::AttributeValue::String(reader) = attr.value() {
                            let s = reader.to_string().unwrap();
                            if s.to_string() == func_name {
                                eprintln!("function {} FOUND!", func_name);
                                flag = true;
                            }
                        } else {
                            println!("read::AttributeValue of this type not handled yet!\n");
                        }
                    } else if attr.name().static_string().unwrap() == "DW_AT_low_pc" {
                        if let read::AttributeValue::Addr(start_addr) = attr.value() {
                            func_start_addr = Some(start_addr);
                            eprintln!("func start addr: {:x}", start_addr);
                        }
                    } else if attr.name().static_string().unwrap() == "DW_AT_high_pc" {
                        if let read::AttributeValue::Udata(end_offset) = attr.value() {
                            func_end_offset = Some(end_offset);
                            eprintln!("func end offset: {}", end_offset);
                        }
                    }
                }
                if flag == true {
                    func_entry_offset = Some(entry.offset());
                    break;
                }
            }
        } else {
            break;
        }
    }
    //let func_entry_offset = func_entry.unwrap().offset();
    (func_entry_offset, func_start_addr, func_end_offset)
}

fn help_infer_generic_type<R: Reader>(_expr: &read::Expression<R>) {}

fn help_infer_read_generic_type<R: Reader>(_expr: &read::Expression<R>) {}

fn help_infer_write_generic_type(_expr: &write::Expression) {}

//let get_addr = |addr: write::Address| -> u64 {
fn get_addr(addr: write::Address) -> u64 {
    match addr {
        write::Address::Constant(value) => {
            //eprintln!("value: {:x}", value);
            value.try_into().unwrap()
        }
        write::Address::Symbol {
            symbol: _, /*ignoring the field*/
            addend,
        } => {
            //eprintln!("addend value: {:x}, symbol: {:?}", addend, symbol);
            addend.try_into().unwrap()
        }
    }
}

fn read_existing_location_lists<R: Reader>(
    dwarf: &read::Dwarf<R>,
    function: &str,
    addresses: &ReadAddressMap,
    var_map: &mut HashMap<String, Vec<(LocationInfo<R>, (Option<u64>, Option<u64>))>>,
    var_empty_scope: &mut HashMap<String, bool>,
    zeroaddr: &mut u64,
)
/* Reading location lists BEGIN */
{
    let (func_entry_offset, func_start_addr, _func_end_offset) =
        get_func_entry_offset(&dwarf, function);
    eprintln!("func_start_addr: {:x}", func_start_addr.unwrap());
    eprintln!("zeroaddr: {:x}", zeroaddr);
    let addr = addresses.get(func_start_addr.unwrap() as usize);
    let value = get_addr(addr);
    eprintln!("actual func_start_addr: {:x}", value);
    *zeroaddr = *zeroaddr - value;

    let units = &mut dwarf.units();
    let unit_header = units.next().unwrap();
    let unit = dwarf.unit(unit_header.unwrap()).unwrap();
    eprintln!("unit low_pc: {}", unit.low_pc);
    let addr = addresses.get(unit.low_pc as usize);
    let unit_low_pc = get_addr(addr);
    eprintln!("actual unit_low_pc = {}", unit_low_pc);

    let mut entries = unit.entries_at_offset(func_entry_offset.unwrap()).unwrap();

    let mut depth = 0;
    let mut first = true;
    let mut low_pc = None;
    let mut high_pc = None;
    let mut scope_ranges: Option<Vec<(u64, u64)>> = None;
    while let Some((index, entry)) = entries.next_dfs().unwrap() {
        depth += index;
        if !first && depth <= 0 {
            break;
        }
        if first == true {
            first = false;
        }
        eprintln!("Entry tag: {:?}", entry.tag().static_string());
        eprintln!("Index : {}", index);
        if true {
            let mut attrs = entry.attrs();
            let mut loclist_vec = Vec::new();
            let mut name = None;
            let mut expr = None;
            let mut expr_present = false;
            let mut write_expr = None;
            let mut write_expr_present = false;
            let mut low_pc_attr_present = false;
            let mut high_pc_attr_present = false;
            let mut ranges_attr_present = false;
            //let mut ss = "";
            while let Some(attr) = attrs.next().unwrap() {
                if attr.name().static_string().unwrap() == "DW_AT_name" {
                    if let read::AttributeValue::DebugStrRef(debug_str_offset) = attr.value() {
                        let str_val = dwarf.string(debug_str_offset).unwrap();
                        eprintln!("Name: {}", str_val.to_string().unwrap());
                        let s = str_val.to_string().unwrap().clone();
                        name = Some(s.into_owned());
                    //name = Some(str_val.to_string().clone().unwrap());
                    } else if let read::AttributeValue::String(reader) = attr.value() {
                        let s = reader.to_string().unwrap();
                        name = Some(s.to_string());
                    } else {
                        eprintln!("read::AttributeValue of this type not handled yet!\n");
                    }
                //eprintln!("Variable Name: {:?}", attr.value());
                } else if attr.name().static_string().unwrap() == "DW_AT_location" {
                    if let read::AttributeValue::LocationListsRef(location_lists_offset) =
                        attr.value()
                    {
                        eprintln!("location lists offset: {:?}", location_lists_offset);
                        let mut loclist_iter =
                            dwarf.locations(&unit, location_lists_offset).unwrap();
                        while let Some(loclist_entry) = loclist_iter.next().unwrap() {
                            /* See -- no need to use actual unit_low_pc ? Seems yes */
                            eprintln!(
                                "loc range: {}, {}",
                                loclist_entry.range.begin - unit.low_pc,
                                loclist_entry.range.end - unit.low_pc
                            );
                            //eprintln!("addresses len + 1: {}", addresses.add(write::Address::Constant(5)));
                            //eprintln!("begin: {:x}, end: {:x}", get_addr(addresses.get(loclist_entry.range.begin as usize)), get_addr(addresses.get(loclist_entry.range.end as usize)));
                            let mut entry = loclist_entry.clone();
                            entry.range.begin -= unit.low_pc;
                            entry.range.end -= unit.low_pc;
                            loclist_vec.push(entry);
                        }
                    } else if let read::AttributeValue::Exprloc(expression) = attr.value() {
                        expr_present = true;
                        expr = Some(expression);
                    } else {
                        eprintln!(
                            "read::AttributeValue -- location -- of this type not handled yet!\n"
                        );
                    }
                } else if attr.name().static_string().unwrap() == "DW_AT_const_value" {
                    write_expr_present = true;
                    let mut error = false;
                    let data = match attr.value() {
                        read::AttributeValue::Data1(data) => data as i64,
                        read::AttributeValue::Data2(data) => data as i64,
                        read::AttributeValue::Data4(data) => data as i64,
                        read::AttributeValue::Data8(data) => data as i64,
                        read::AttributeValue::Sdata(data) => data as i64,
                        read::AttributeValue::Udata(data) => data as i64,
                        _ => {
                            error = true;
                            0i64
                        }
                    };
                    if error {
                        eprintln!("ERROR: Invalid data in DW_AT_const_value!");
                    }
                    let mut expression = write::Expression::new();
                    expression.op_consts(data);
                    expression.op(DW_OP_stack_value);
                    write_expr = Some(expression);

                /*if low_pc != None && high_pc != None {
                    eprintln!("low_pc and high_pc available!");
                    let low_pc = low_pc.unwrap();
                    let high_pc = high_pc.unwrap();
                    let high_pc = low_pc + high_pc;
                }*/
                } else if attr.name().static_string().unwrap() == "DW_AT_low_pc" {
                    if let read::AttributeValue::Addr(_addr) = attr.value() {
                        low_pc_attr_present = true;
                        /*low_pc = Some(addr);
                        eprintln!("low_pc := {}", addr);*/
                    }
                } else if attr.name().static_string().unwrap() == "DW_AT_high_pc" {
                    if let read::AttributeValue::Udata(_offset) = attr.value() {
                        high_pc_attr_present = true;
                    /*high_pc = Some(offset);
                    eprintln!("high_pc := {}", offset);*/
                    } else {
                        eprintln!("ERROR: Expected offset in DW_AT_high_pc!");
                    }
                } else if attr.name().static_string().unwrap() == "DW_AT_ranges" {
                    if let read::AttributeValue::RangeListsRef(_offset) = attr.value() {
                        ranges_attr_present = true;
                    }
                }
            }

            if entry.tag() == gimli::DW_TAG_formal_parameter
                || entry.tag() == gimli::DW_TAG_variable
            {
                if name != None {
                    /* Multiple variables with same name not handled in this var_empty_scope map
                     * yet! */
                    if scope_ranges == None {
                        var_empty_scope.insert(name.clone().unwrap(), true);
                    } else {
                        var_empty_scope.insert(name.clone().unwrap(), false);
                    }
                }
            }

            let low_pc_val = entry.attr_value(gimli::DW_AT_low_pc).unwrap();
            let high_pc_val = entry.attr_value(gimli::DW_AT_high_pc).unwrap();
            if low_pc_attr_present && high_pc_attr_present {
                let low_pc_val = low_pc_val.unwrap();
                let high_pc_val = high_pc_val.unwrap();
                if let read::AttributeValue::Addr(addr) = low_pc_val {
                    low_pc = Some(addr);
                    eprintln!("low_pc := {}", addr);
                }
                if let read::AttributeValue::Udata(offset) = high_pc_val {
                    high_pc = Some(offset);
                    eprintln!("high_pc := {}", offset);
                }
                if low_pc != None && high_pc != None {
                    let low_pc = low_pc.unwrap();
                    let low_pc = get_addr(addresses.get(low_pc as usize));
                    let high_pc = low_pc + high_pc.unwrap();
                    let scope_vec = vec![(low_pc, high_pc)];
                    scope_ranges = Some(scope_vec);
                }
            }

            let ranges_val = entry.attr_value(gimli::DW_AT_ranges).unwrap();
            if ranges_attr_present {
                let ranges_val = ranges_val.unwrap();
                if let read::AttributeValue::RangeListsRef(offset) = ranges_val {
                    let mut rangelist_iter = dwarf.ranges(&unit, offset).unwrap();
                    let mut scope_vec = Vec::new();
                    while let Some(range_entry) = rangelist_iter.next().unwrap() {
                        let begin =
                            get_addr(addresses.get((range_entry.begin - unit.low_pc) as usize));
                        let end = get_addr(addresses.get((range_entry.end - unit.low_pc) as usize));
                        scope_vec.push((begin, end));
                    }
                    if scope_vec.is_empty() {
                        scope_ranges = None;
                    } else {
                        scope_ranges = Some(scope_vec);
                    }
                }
            }

            if name != None {
                if !loclist_vec.is_empty() {
                    eprintln!("Trying to add a LocList..");
                    if let Some(v) = var_map.get_mut(&name.clone().unwrap()) {
                        /* Look at the semantics of high_pc (whether it's just an offset or the
                         * actual address?) throughout the file -- try to keep it
                         * uniform */
                        v.push((LocationInfo::LocList(loclist_vec), (low_pc, high_pc)));
                    } else {
                        var_map.insert(
                            name.unwrap(),
                            vec![(LocationInfo::LocList(loclist_vec), (low_pc, high_pc))],
                        );
                    }
                } else if low_pc != None && high_pc != None {
                    let low_pc = low_pc.unwrap();
                    let high_pc = high_pc.unwrap();
                    //let high_pc = low_pc + high_pc;
                    if expr_present || write_expr_present {
                        if expr_present {
                            let location = Location {
                                expression: expr.unwrap(),
                                begin: low_pc,
                                end: high_pc,
                            };
                            eprintln!("Trying to add a Loc..");
                            if let Some(v) = var_map.get_mut(&name.clone().unwrap()) {
                                /* Look at the semantics of high_pc (whether it's just an offset or the
                                 * actual address?) throughout the file -- try to keep it
                                 * uniform */
                                v.push((
                                    LocationInfo::Loc(location),
                                    (Some(low_pc), Some(high_pc)),
                                ));
                            } else {
                                var_map.insert(
                                    name.unwrap(),
                                    vec![(
                                        LocationInfo::Loc(location),
                                        (Some(low_pc), Some(high_pc)),
                                    )],
                                );
                            }
                        } else
                        /*if write_expr_present*/
                        {
                            let location = WriteLocation {
                                expression: write_expr.unwrap(),
                                begin: low_pc,
                                /* see if high_pc is right */
                                end: high_pc,
                            };
                            eprintln!("write_expr_present: begin = {}, end = {}", low_pc, high_pc);
                            eprintln!("Trying to add a WLoc..");
                            if let Some(v) = var_map.get_mut(&name.clone().unwrap()) {
                                /* Look at the semantics of high_pc (whether it's just an offset or the
                                 * actual address?) throughout the file -- try to keep it
                                 * uniform */
                                v.push((
                                    LocationInfo::WLoc(location),
                                    (Some(low_pc), Some(high_pc)),
                                ));
                            } else {
                                var_map.insert(
                                    name.unwrap(),
                                    vec![(
                                        LocationInfo::WLoc(location),
                                        (Some(low_pc), Some(high_pc)),
                                    )],
                                );
                            }
                        }
                    } else {
                        /* Make sure to iterate correctly over the IntLocList, whenever we get a
                         * value from var_map, as we are adding an
                         * empty list here */
                        let location_list = Vec::new();
                        if let Some(v) = var_map.get_mut(&name.clone().unwrap()) {
                            /* Look at the semantics of high_pc (whether it's just an offset or the
                             * actual address?) throughout the file -- try to keep it
                             * uniform */
                            v.push((
                                LocationInfo::IntLocList(location_list),
                                (Some(low_pc), Some(high_pc)),
                            ));
                        } else {
                            var_map.insert(
                                name.unwrap(),
                                vec![(
                                    LocationInfo::IntLocList(location_list),
                                    (Some(low_pc), Some(high_pc)),
                                )],
                            );
                        }
                    }
                }
            }
        }
    }
}
/* Reading location lists END */

pub fn rewrite_dwarf(
    in_object: &object::File<'_>,
    out_object: &mut object_write::Object,
    symbols: &HashMap<SymbolIndex, object_write::SymbolId>,
) {
    /*
    // Define the sections we can't convert yet.
    for section in in_object.sections() {
        if let Some(name) = section.name() {
            if !is_rewrite_dwarf_section(&section) {
                artifact.declare(name, Decl::debug_section()).unwrap();
                artifact
                    .define(name, section.uncompressed_data().into_owned())
                    .unwrap();
            }
        }
    }
    */

    fn get_reader<'a>(
        data: &'a [u8],
        relocations: &'a ReadRelocationMap,
        addresses: &'a ReadAddressMap,
    ) -> ReaderRelocate<'a, EndianSlice<'a, LittleEndian>> {
        let section = EndianSlice::new(data, LittleEndian);
        let reader = section.clone();
        ReaderRelocate {
            relocations,
            addresses,
            section,
            reader,
        }
    };

    let addresses = ReadAddressMap::default();
    let no_section = (Cow::Borrowed(&[][..]), ReadRelocationMap::default());
    let (debug_abbrev_data, debug_abbrev_relocs) = get_section(in_object, ".debug_abbrev");
    let (debug_addr_data, debug_addr_relocs) = get_section(in_object, ".debug_addr");
    let (debug_info_data, debug_info_relocs) = get_section(in_object, ".debug_info");
    let (debug_line_data, debug_line_relocs) = get_section(in_object, ".debug_line");
    let (debug_line_str_data, debug_line_str_relocs) = get_section(in_object, ".debug_line_str");
    let (debug_loc_data, debug_loc_relocs) = get_section(in_object, ".debug_loc");
    let (debug_loclists_data, debug_loclists_relocs) = get_section(in_object, ".debug_loclists");
    let (debug_ranges_data, debug_ranges_relocs) = get_section(in_object, ".debug_ranges");
    /*let (debug_aranges_data, debug_aranges_relocs) = get_section(in_object, ".debug_aranges");*/
    let (debug_rnglists_data, debug_rnglists_relocs) = get_section(in_object, ".debug_rnglists");
    let (debug_str_data, debug_str_relocs) = get_section(in_object, ".debug_str");
    let (debug_str_offsets_data, debug_str_offsets_relocs) =
        get_section(in_object, ".debug_str_offsets");
    let (debug_types_data, debug_types_relocs) = get_section(in_object, ".debug_types");
    let dwarf = read::Dwarf {
        debug_abbrev: read::DebugAbbrev::from(get_reader(
            &debug_abbrev_data,
            &debug_abbrev_relocs,
            &addresses,
        )),
        debug_addr: read::DebugAddr::from(get_reader(
            &debug_addr_data,
            &debug_addr_relocs,
            &addresses,
        )),
        debug_info: read::DebugInfo::from(get_reader(
            &debug_info_data,
            &debug_info_relocs,
            &addresses,
        )),
        debug_line: read::DebugLine::from(get_reader(
            &debug_line_data,
            &debug_line_relocs,
            &addresses,
        )),
        debug_line_str: read::DebugLineStr::from(get_reader(
            &debug_line_str_data,
            &debug_line_str_relocs,
            &addresses,
        )),
        debug_str: read::DebugStr::from(get_reader(&debug_str_data, &debug_str_relocs, &addresses)),
        debug_str_offsets: read::DebugStrOffsets::from(get_reader(
            &debug_str_offsets_data,
            &debug_str_offsets_relocs,
            &addresses,
        )),
        debug_str_sup: read::DebugStr::from(get_reader(&no_section.0, &no_section.1, &addresses)),
        debug_types: read::DebugTypes::from(get_reader(
            &debug_types_data,
            &debug_types_relocs,
            &addresses,
        )),
        locations: read::LocationLists::new(
            read::DebugLoc::from(get_reader(&debug_loc_data, &debug_loc_relocs, &addresses)),
            read::DebugLocLists::from(get_reader(
                &debug_loclists_data,
                &debug_loclists_relocs,
                &addresses,
            )),
        ),
        ranges: read::RangeLists::new(
            read::DebugRanges::from(get_reader(
                &debug_ranges_data,
                &debug_ranges_relocs,
                &addresses,
            )),
            read::DebugRngLists::from(get_reader(
                &debug_rnglists_data,
                &debug_rnglists_relocs,
                &addresses,
            )),
        ),
    };
    /*
    let (eh_frame_data, eh_frame_relocs) = get_section(in_object, ".eh_frame");
    let eh_frame = read::EhFrame::from(get_reader(&eh_frame_data, &eh_frame_relocs, &addresses));
    */
    /*let debug_aranges = read::DebugAranges::from(get_reader(
                          &debug_aranges_data,
                          &debug_aranges_relocs,
                          &addresses,
    ));*/

    let mut buffer = String::new();
    std::io::stdin().read_to_string(&mut buffer).unwrap();
    //println!("{}", buffer);
    let mut lines = buffer.as_str().lines();
    let line = lines.next().unwrap();
    //println!("first line: {}", first_line);
    //let function: Vec<&str> = first_line.split(':').collect();
    //let function = function[1];
    assert!(line == "=ZeroAddress", "Invalid expr-file-format!");
    let line = lines.next().unwrap();
    let zeroaddr = line;
    let zeroaddr = zeroaddr.trim_start_matches("0x");
    let mut zeroaddr = u64::from_str_radix(zeroaddr, 16).unwrap();
    let line = lines.next().unwrap();
    assert!(line == "=TotalPCs", "Invalid expr-file-format!");
    let line = lines.next().unwrap();
    let _total_pcs = line.parse::<u32>().unwrap();
    let line = lines.next().unwrap();
    assert!(line == "=Function", "Invalid expr-file-format!");
    let line = lines.next().unwrap();
    let function = line;
    println!("[LOG]: function: {}", function);

    let mut var_map = HashMap::new();
    let mut var_empty_scope = HashMap::new();

    read_existing_location_lists(
        &dwarf,
        &function,
        &addresses,
        &mut var_map,
        &mut var_empty_scope,
        &mut zeroaddr,
    );

    //REMOVE this
    //eprintln!("addresses len + 1: {}", addresses.add(write::Address::Constant(5)));

    let convert_address = |index| Some(addresses.get(index as usize));

    let mut dwarf = match write::Dwarf::from(&dwarf, &convert_address) {
        Ok(dwarf) => dwarf,
        Err(write::ConvertError::Read(err)) => {
            eprintln!("dwarf convert: {}", dwarf.format_error(err));
            panic!();
        }
        _ => panic!(),
    };

    //let test = 130;
    //eprintln!("testaddr: {:x}", get_addr(addresses.get(test as usize)));
    //REMOVE this
    //eprintln!("addresses len + 1: {}", addresses.add(write::Address::Constant(5)));

    let line = lines.next().unwrap();
    assert!(line == "=Expressions", "Invalid expr-file-format!");
    for line in lines {
        //print!("{}", line);
        //print!("\n");
        let pc_var_expr: Vec<&str> = line.split('\t').collect();
        let pc_range: Vec<&str> = pc_var_expr[1].split("->").collect();
        let var_and_expr: Vec<&str> = pc_var_expr[0].split('=').collect();
        let mut var_name = var_and_expr[0];
        let loc_expr = var_and_expr[1];
        if var_name.contains("(")
            || loc_expr.contains("(")
            || var_name.contains("phi")
            || var_name.starts_with("symbol")
            || var_name.starts_with("%")
            || var_name.starts_with("input.dst.")
            || var_name.starts_with("input.src.")
        {
            println!("[LOG]: Skipping {}: {}", var_and_expr[0], var_and_expr[1]);
            continue;
        }
        println!("[LOG]: Processing {}: {}", var_and_expr[0], var_and_expr[1]);
        if var_name.contains(".") {
            let tokens: Vec<&str> = var_name.split('.').collect();
            var_name = tokens[0];
        }
        println!("[LOG]: varname: {}", var_name);

        let units = &mut dwarf.units;
        let strings = &mut dwarf.strings;
        let unit = units.get_mut(units.id(0));
        let func_id = get_func_id(unit, strings, function); //.expect("Provided function not present in .debug_info!");
        if func_id == None {
            println!(
                "[LOG]: Provided function: {} not present in .debug_info! Skipping..",
                function
            );
            continue;
        }
        let begin = pc_range[0].trim_start_matches("0x");
        let begin = u64::from_str_radix(begin, 16).unwrap();
        let end = pc_range[1].trim_start_matches("0x");
        let end = u64::from_str_radix(end, 16).unwrap();

        let mut start = begin - zeroaddr;
        let mut end = end - zeroaddr;

        let key_present = var_map.contains_key(var_name);
        println!("key_present = {}", key_present);
        //assert!(key_present == true, "var_name not present in var_map!");
        if key_present == false {
            eprintln!(
                "[LOG]: Variable: {} not present in the var_map! Skipping..",
                var_name
            );
            continue;
        }
        let no_of_vars = var_map.get(&var_name.to_string()).unwrap().len();

        let (var, parent) = get_var(
            unit,
            &func_id.unwrap(),
            strings,
            var_name,
            (start, end),
            no_of_vars as u8,
        ); //.expect("Variable not present in the lexical block!");
        if var == None {
            println!(
                "[LOG]: Variable: {} not present in the function! Skipping..",
                var_name
            );
            continue;
        }
        if no_of_vars == 1 {
            let vec_loc_info = var_map.get(&var_name.to_string()).unwrap();
            let (_, (var_low, var_high)) = vec_loc_info[0];

            let var_low = var_low.unwrap();
            let var_low = get_addr(addresses.get(var_low as usize));
            let var_high = var_high.unwrap();
            eprintln!(
                "start: {}, end: {}, var_low: {}, var_low+var_high: {}",
                start,
                end,
                var_low,
                var_low + var_high
            );
            if start < var_low {
                start = var_low;
            }
            if end > var_low + var_high {
                end = var_low + var_high;
            }
            assert!(start < end, "Error: start >= end!");
        }

        let new_dwarf_expr = create_dwarf_expr(loc_expr);

        let mut new_loc_list = Vec::new();
        let encoding = unit.encoding();

        let mut process_location = |data, locbegin: u64, locend: u64| {
            /* Note: Don't remove below line - it is used to circumvent adding a new function
             * instead of a closure and it's allowing type inference for the parameter 'data'.
             * This has to do with Closures with Generics. So far, there is no way to specify
             * generic type as part of a closure. */
            eprintln!("process_location BEGIN");
            help_infer_generic_type(data);

            let rangebegin = locbegin as usize;
            let rangeend = locend as usize;
            let rangebegin = get_addr(addresses.get(rangebegin));
            let rangeend = get_addr(addresses.get(rangeend));

            let write_expr =
                write::Expression::from(data.clone(), encoding, None, None, None, &convert_address);
            let write_expr = write_expr.unwrap();

            if (start < rangebegin && end <= rangebegin) || (start >= rangeend && end > rangeend) {
                // do as normal
                eprintln!("Do as normal");
                eprintln!(
                    "start: {:x}, end: {:x}, rangebegin: {:x}, rangeend: {:x}",
                    start, end, rangebegin, rangeend
                );

                let rangebegin = locbegin as usize;
                let rangeend = locend as usize;

                let new_loc = write::Location::StartEnd {
                    begin: addresses.get(rangebegin),
                    end: addresses.get(rangeend),
                    data: write_expr,
                };
                new_loc_list.push(new_loc);
            } else if start <= rangebegin && end >= rangeend {
                // ignore this range
                eprintln!("Ignore this change");
            } else if start >= rangebegin && end <= rangeend {
                if start == rangebegin
                /* && end < rangeend */
                {
                    // split into two
                    // start-end, end-rangeend
                    eprintln!("Split into two");

                    //let rangebegin = locbegin as usize;
                    let rangeend = locend as usize;

                    let new_loc = write::Location::StartEnd {
                        begin: write::Address::Constant(end),
                        end: addresses.get(rangeend),
                        data: write_expr,
                    };
                    new_loc_list.push(new_loc);
                } else if end == rangeend
                /* && start > rangebegin */
                {
                    // split into two
                    // rangebegin-start, start-end
                    eprintln!("Split into two");

                    let rangebegin = locbegin as usize;
                    //let rangeend = locend as usize;

                    let new_loc = write::Location::StartEnd {
                        begin: addresses.get(rangebegin),
                        end: write::Address::Constant(start),
                        data: write_expr,
                    };
                    new_loc_list.push(new_loc);
                } else {
                    // split into three
                    // rangebegin-start, start-end, end-rangeend
                    eprintln!("Split into three");
                    eprintln!(
                        "start: {:x}, end: {:x}, rangebegin: {:x}, rangeend: {:x}",
                        start, end, rangebegin, rangeend
                    );

                    let rangebegin = locbegin as usize;
                    let rangeend = locend as usize;

                    let new_loc = write::Location::StartEnd {
                        begin: addresses.get(rangebegin),
                        end: write::Address::Constant(start),
                        data: write_expr.clone(),
                    };
                    new_loc_list.push(new_loc);
                    let new_loc = write::Location::StartEnd {
                        begin: write::Address::Constant(end),
                        end: addresses.get(rangeend),
                        data: write_expr,
                    };
                    new_loc_list.push(new_loc);
                }
            } else if start < rangebegin && end >= rangebegin && end < rangeend {
                // split into two
                // start-end, end-rangeend
                eprintln!("Split into two");
                //let rangebegin = locbegin as usize;
                let rangeend = locend as usize;
                let new_loc = write::Location::StartEnd {
                    begin: write::Address::Constant(end),
                    end: addresses.get(rangeend),
                    data: write_expr,
                };
                new_loc_list.push(new_loc);
            } else if end > rangeend && start > rangebegin && start < rangeend {
                // split into two
                // rangebegin-start, start-end
                eprintln!("Split into two");
                let rangebegin = locbegin as usize;
                //let rangeend = locend as usize;
                let new_loc = write::Location::StartEnd {
                    begin: addresses.get(rangebegin),
                    end: write::Address::Constant(start),
                    data: write_expr,
                };
                new_loc_list.push(new_loc);
            } else {
                // unreachable
                assert!(false, "Unreachable block executed!");
            }
            eprintln!("process_location END");
        };
        /* Rust tricks - to separate out the gimli::read::Expression and gimli::write::Expression,
         * had to create a new closure - otherwise, the type inferencing doesn't work -
         * Additionally, separating them into two based on read::Expression and loclist and
         * single location */
        let mut new_read_loc_list = Vec::new();
        let mut process_read_location = |data, locbegin: u64, locend: u64| {
            /* Note: Don't remove below line - it is used to circumvent adding a new function
             * instead of a closure and it's allowing type inference for the parameter 'data'.
             * This has to do with Closures with Generics. So far, there is no way to specify
             * generic type as part of a closure. */
            help_infer_read_generic_type(data);

            let rangebegin = locbegin as usize;
            let rangeend = locend as usize;
            let rangebegin = get_addr(addresses.get(rangebegin));
            //let rangeend = get_addr(addresses.get(rangeend));
            //let rangebegin = rangebegin as u64;
            let rangeend = rangeend as u64;

            let write_expr =
                write::Expression::from(data.clone(), encoding, None, None, None, &convert_address);
            let write_expr = write_expr.unwrap();

            if (start < rangebegin && end <= rangebegin) || (start >= rangeend && end > rangeend) {
                // do as normal
                eprintln!("Do as normal");
                eprintln!(
                    "start: {:x}, end: {:x}, rangebegin: {:x}, rangeend: {:x}",
                    start, end, rangebegin, rangeend
                );

                let rangebegin = locbegin as usize;
                let rangeend = locend as usize;

                let new_loc = write::Location::StartEnd {
                    begin: addresses.get(rangebegin),
                    end: write::Address::Constant(rangeend as u64),
                    data: write_expr,
                };
                new_read_loc_list.push(new_loc);
            } else if start <= rangebegin && end >= rangeend {
                // ignore this range
                eprintln!("Ignore this change");
            } else if start >= rangebegin && end <= rangeend {
                if start == rangebegin
                /* && end < rangeend */
                {
                    // split into two
                    // start-end, end-rangeend
                    eprintln!("Split into two");

                    //let rangebegin = locbegin as usize;
                    let rangeend = locend as usize;

                    let new_loc = write::Location::StartEnd {
                        begin: write::Address::Constant(end),
                        end: write::Address::Constant(rangeend as u64),
                        data: write_expr,
                    };
                    new_read_loc_list.push(new_loc);
                } else if end == rangeend
                /* && start > rangebegin */
                {
                    // split into two
                    // rangebegin-start, start-end
                    eprintln!("Split into two");

                    let rangebegin = locbegin as usize;
                    //let rangeend = locend as usize;

                    let new_loc = write::Location::StartEnd {
                        begin: addresses.get(rangebegin),
                        end: write::Address::Constant(start),
                        data: write_expr,
                    };
                    new_read_loc_list.push(new_loc);
                } else {
                    // split into three
                    // rangebegin-start, start-end, end-rangeend
                    eprintln!("Split into three");
                    eprintln!(
                        "start: {:x}, end: {:x}, rangebegin: {:x}, rangeend: {:x}",
                        start, end, rangebegin, rangeend
                    );

                    let rangebegin = locbegin as usize;
                    let rangeend = locend as usize;

                    let new_loc = write::Location::StartEnd {
                        begin: addresses.get(rangebegin),
                        end: write::Address::Constant(start),
                        data: write_expr.clone(),
                    };
                    new_read_loc_list.push(new_loc);
                    let new_loc = write::Location::StartEnd {
                        begin: write::Address::Constant(end),
                        end: write::Address::Constant(rangeend as u64),
                        data: write_expr,
                    };
                    new_read_loc_list.push(new_loc);
                }
            } else if start < rangebegin && end >= rangebegin && end < rangeend {
                // split into two
                // start-end, end-rangeend
                eprintln!("Split into two");
                //let rangebegin = locbegin as usize;
                let rangeend = locend as usize;
                let new_loc = write::Location::StartEnd {
                    begin: write::Address::Constant(end),
                    end: write::Address::Constant(rangeend as u64),
                    data: write_expr,
                };
                new_read_loc_list.push(new_loc);
            } else if end > rangeend && start > rangebegin && start < rangeend {
                // split into two
                // rangebegin-start, start-end
                eprintln!("Split into two");
                let rangebegin = locbegin as usize;
                //let rangeend = locend as usize;
                let new_loc = write::Location::StartEnd {
                    begin: addresses.get(rangebegin),
                    end: write::Address::Constant(start),
                    data: write_expr,
                };
                new_read_loc_list.push(new_loc);
            } else {
                // unreachable
                assert!(false, "Unreachable block executed!");
            }
        };
        /* Rust tricks - to separate out the gimli::read::Expression and gimli::write::Expression,
         * had to create a new closure - otherwise, the type inferencing doesn't work */
        let mut new_write_loc_list = Vec::new();
        let mut process_write_location = |data, locbegin: u64, locend: u64| {
            /* Note: Don't remove below line - it is used to circumvent adding a new function
             * instead of a closure and it's allowing type inference for the parameter 'data'.
             * This has to do with Closures with Generics. So far, there is no way to specify
             * generic type as part of a closure. */
            help_infer_write_generic_type(data);

            let rangebegin = locbegin as usize;
            let rangeend = locend as usize;
            let rangebegin = get_addr(addresses.get(rangebegin));
            //let rangeend = get_addr(addresses.get(rangeend));
            //let rangebegin = rangebegin as u64;
            let rangeend = rangeend as u64;

            let write_expr = data.clone();

            if (start < rangebegin && end <= rangebegin) || (start >= rangeend && end > rangeend) {
                // do as normal
                eprintln!("Do as normal");
                eprintln!(
                    "start: {:x}, end: {:x}, rangebegin: {:x}, rangeend: {:x}",
                    start, end, rangebegin, rangeend
                );

                let rangebegin = locbegin as usize;
                let rangeend = locend as usize;

                let new_loc = write::Location::StartEnd {
                    begin: addresses.get(rangebegin),
                    end: write::Address::Constant(rangeend as u64),
                    data: write_expr,
                };
                new_write_loc_list.push(new_loc);
            } else if start <= rangebegin && end >= rangeend {
                // ignore this range
                eprintln!("Ignore this change");
            } else if start >= rangebegin && end <= rangeend {
                if start == rangebegin
                /* && end < rangeend */
                {
                    // split into two
                    // start-end, end-rangeend
                    eprintln!("Split into two");

                    //let rangebegin = locbegin as usize;
                    let rangeend = locend as usize;

                    let new_loc = write::Location::StartEnd {
                        begin: write::Address::Constant(end),
                        end: write::Address::Constant(rangeend as u64),
                        data: write_expr,
                    };
                    new_write_loc_list.push(new_loc);
                } else if end == rangeend
                /* && start > rangebegin */
                {
                    // split into two
                    // rangebegin-start, start-end
                    eprintln!("Split into two");

                    let rangebegin = locbegin as usize;
                    //let rangeend = locend as usize;

                    let new_loc = write::Location::StartEnd {
                        begin: addresses.get(rangebegin),
                        end: write::Address::Constant(start),
                        data: write_expr,
                    };
                    new_write_loc_list.push(new_loc);
                } else {
                    // split into three
                    // rangebegin-start, start-end, end-rangeend
                    eprintln!("Split into three");
                    eprintln!(
                        "start: {:x}, end: {:x}, rangebegin: {:x}, rangeend: {:x}",
                        start, end, rangebegin, rangeend
                    );

                    let rangebegin = locbegin as usize;
                    let rangeend = locend as usize;

                    let new_loc = write::Location::StartEnd {
                        begin: addresses.get(rangebegin),
                        end: write::Address::Constant(start),
                        data: write_expr.clone(),
                    };
                    new_write_loc_list.push(new_loc);
                    let new_loc = write::Location::StartEnd {
                        begin: write::Address::Constant(end),
                        end: write::Address::Constant(rangeend as u64),
                        data: write_expr,
                    };
                    new_write_loc_list.push(new_loc);
                }
            } else if start < rangebegin && end >= rangebegin && end < rangeend {
                // split into two
                // start-end, end-rangeend
                eprintln!("Split into two");
                //let rangebegin = locbegin as usize;
                let rangeend = locend as usize;
                let new_loc = write::Location::StartEnd {
                    begin: write::Address::Constant(end),
                    end: write::Address::Constant(rangeend as u64),
                    data: write_expr,
                };
                new_write_loc_list.push(new_loc);
            } else if end > rangeend && start > rangebegin && start < rangeend {
                // split into two
                // rangebegin-start, start-end
                eprintln!("Split into two");
                let rangebegin = locbegin as usize;
                //let rangeend = locend as usize;
                let new_loc = write::Location::StartEnd {
                    begin: addresses.get(rangebegin),
                    end: write::Address::Constant(start),
                    data: write_expr,
                };
                new_write_loc_list.push(new_loc);
            } else {
                // unreachable
                assert!(false, "Unreachable block executed!");
            }
        };
        let mut new_int_write_loc_list = Vec::new();
        let mut process_int_write_location =
            |data, locbegin: write::Address, locend: write::Address| {
                /* Note: Don't remove below line - it is used to circumvent adding a new function
                 * instead of a closure and it's allowing type inference for the parameter 'data'.
                 * This has to do with Closures with Generics. So far, there is no way to specify
                 * generic type as part of a closure. */
                help_infer_write_generic_type(data);

                let rangebegin = get_addr(locbegin);
                let rangeend = get_addr(locend);

                let write_expr = data.clone();

                if (start < rangebegin && end <= rangebegin)
                    || (start >= rangeend && end > rangeend)
                {
                    // do as normal
                    eprintln!("Do as normal");
                    eprintln!(
                        "start: {:x}, end: {:x}, rangebegin: {:x}, rangeend: {:x}",
                        start, end, rangebegin, rangeend
                    );

                    let new_loc = write::Location::StartEnd {
                        begin: locbegin,
                        end: locend,
                        data: write_expr,
                    };
                    new_int_write_loc_list.push(new_loc);
                } else if start <= rangebegin && end >= rangeend {
                    // ignore this range
                    eprintln!("Ignore this change");
                } else if start >= rangebegin && end <= rangeend {
                    if start == rangebegin
                    /* && end < rangeend */
                    {
                        // split into two
                        // start-end, end-rangeend
                        eprintln!("Split into two");

                        let new_loc = write::Location::StartEnd {
                            begin: write::Address::Constant(end),
                            end: locend,
                            data: write_expr,
                        };
                        new_int_write_loc_list.push(new_loc);
                    } else if end == rangeend
                    /* && start > rangebegin */
                    {
                        // split into two
                        // rangebegin-start, start-end
                        eprintln!("Split into two");

                        let new_loc = write::Location::StartEnd {
                            begin: locbegin,
                            end: write::Address::Constant(start),
                            data: write_expr,
                        };
                        new_int_write_loc_list.push(new_loc);
                    } else {
                        // split into three
                        // rangebegin-start, start-end, end-rangeend
                        eprintln!("Split into three");
                        eprintln!(
                            "start: {:x}, end: {:x}, rangebegin: {:x}, rangeend: {:x}",
                            start, end, rangebegin, rangeend
                        );

                        let new_loc = write::Location::StartEnd {
                            begin: locbegin,
                            end: write::Address::Constant(start),
                            data: write_expr.clone(),
                        };
                        new_int_write_loc_list.push(new_loc);
                        let new_loc = write::Location::StartEnd {
                            begin: write::Address::Constant(end),
                            end: locend,
                            data: write_expr,
                        };
                        new_int_write_loc_list.push(new_loc);
                    }
                } else if start < rangebegin && end >= rangebegin && end < rangeend {
                    // split into two
                    // start-end, end-rangeend
                    eprintln!("Split into two");

                    let new_loc = write::Location::StartEnd {
                        begin: write::Address::Constant(end),
                        end: locend,
                        data: write_expr,
                    };
                    new_int_write_loc_list.push(new_loc);
                } else if end > rangeend && start > rangebegin && start < rangeend {
                    // split into two
                    // rangebegin-start, start-end
                    eprintln!("Split into two");

                    let new_loc = write::Location::StartEnd {
                        begin: locbegin,
                        end: write::Address::Constant(start),
                        data: write_expr,
                    };
                    new_int_write_loc_list.push(new_loc);
                } else {
                    // unreachable
                    assert!(false, "Unreachable block executed!");
                }
            };
        let mut const_value_attr = false;
        let mut read_location = false;
        let mut int_loc_list = false;
        let mut pc_range = (None, None);
        /*if loclist_vec != None {}*/
        /* can't be compared !! */
        if key_present {
            let vec_loc_info = var_map.get_mut(&var_name.to_string()).unwrap();
            let (loc_info, pc_range_tmp) = {
                let mut curr_pc_range: (Option<u64>, Option<u64>) = (None, None);
                let mut curr_loc_info = None;
                let mut curr_i = None;
                for (i, (loc_info, pc_range_tmp)) in vec_loc_info.iter().enumerate() {
                    println!("vec_loc_info iter..");
                    if let (Some(l), Some(h)) = pc_range_tmp {
                        eprintln!("Found some l and h! l: {} h: {}", l, h);
                        eprintln!("start: {}, end: {}", start, end);
                        let l = get_addr(addresses.get(*l as usize));
                        eprintln!("New l: {}", l);
                        if start >= l && end <= l + h {
                            if let (Some(curr_l), Some(curr_h)) = curr_pc_range {
                                let curr_l = get_addr(addresses.get(curr_l as usize));
                                if l >= curr_l && (l + h <= curr_l + curr_h) {
                                    curr_pc_range = (pc_range_tmp.0, pc_range_tmp.1);
                                    curr_loc_info = Some((*loc_info).clone());
                                    curr_i = Some(i);
                                }
                            } else
                            /* if curr_pc_range == (None, None) */
                            {
                                curr_pc_range = (pc_range_tmp.0, pc_range_tmp.1);
                                curr_loc_info = Some((*loc_info).clone());
                                curr_i = Some(i);
                            }
                        } else {
                            eprintln!("l and h didn't satisfy constraints!");
                        }
                    }
                }
                if let Some(curr_i) = curr_i {
                    vec_loc_info.remove(curr_i);
                }
                (curr_loc_info, curr_pc_range)
            };
            /*
            let vec_loc_info = var_map.remove(var_name);
            let (loc_info, pc_range_tmp) = &vec_loc_info.unwrap()[0];
            */

            if let Some(loc_info) = loc_info {
                pc_range = pc_range_tmp;
                eprintln!("Calling one of the process_locations..");
                if let LocationInfo::LocList(loclist_vec) = loc_info {
                    for (_i, loclist_entry) in loclist_vec.iter().enumerate() {
                        process_location(
                            &loclist_entry.data,
                            loclist_entry.range.begin,
                            loclist_entry.range.end,
                        );
                    }
                } else if let LocationInfo::Loc(location) = loc_info {
                    process_read_location(
                        &location.expression,
                        location.begin,
                        /* See if below line is correct -- it corresponds to use of low_pc + high_pc or
                         * just high_pc somewhere above in the code*/
                        get_addr(addresses.get(location.begin as usize)) + location.end,
                    );
                    read_location = true;
                } else if let LocationInfo::WLoc(location) = loc_info {
                    process_write_location(
                        &location.expression,
                        location.begin,
                        /* See if below line is correct -- it corresponds to use of low_pc + high_pc or
                         * just high_pc somewhere above in the code*/
                        get_addr(addresses.get(location.begin as usize)) + location.end,
                    );
                    const_value_attr = true;
                } else if let LocationInfo::IntLocList(loclist_vec) = loc_info {
                    for loc_entry in loclist_vec.iter() {
                        if let write::Location::StartEnd { begin, end, data } = loc_entry {
                            /*let mut b = None;
                            let mut e = None;
                            if let write::Address::Constant(begin_const) = begin {
                                b = Some(*begin_const);
                            }
                            if let write::Address::Constant(end_const) = end {
                                e = Some(*end_const);
                            }
                            if b != None && e != None {
                                process_int_write_location(
                                    &data,
                                    b.unwrap(),
                                    e.unwrap(),
                                );
                            }*/
                            process_int_write_location(&data, *begin, *end);
                        }
                    }
                    int_loc_list = true;
                }
            }
        }

        let new_loc = gimli::write::Location::StartEnd {
            begin: write::Address::Constant(start),
            end: write::Address::Constant(end),
            data: new_dwarf_expr,
        };
        /* Rust tricks - to separate out the mutable new_loc_list, using two different loc_lists
         * for read and write */
        if const_value_attr {
            println!("got into new_write_loc_list..");
            new_write_loc_list.push(new_loc);
            if let Some(v) = var_map.get_mut(&var_name.to_string()) {
                v.push((
                    LocationInfo::IntLocList(new_write_loc_list.clone()),
                    pc_range,
                ));
            } else {
                var_map.insert(
                    var_name.to_string(),
                    vec![(
                        LocationInfo::IntLocList(new_write_loc_list.clone()),
                        pc_range,
                    )],
                );
            }
        } else if read_location {
            println!("got into new_read_loc_list..");
            new_read_loc_list.push(new_loc);
            if let Some(v) = var_map.get_mut(&var_name.to_string()) {
                v.push((
                    LocationInfo::IntLocList(new_read_loc_list.clone()),
                    pc_range,
                ));
            } else {
                var_map.insert(
                    var_name.to_string(),
                    vec![(
                        LocationInfo::IntLocList(new_read_loc_list.clone()),
                        pc_range,
                    )],
                );
            }
        } else if int_loc_list {
            println!("got into int_loc_list..");
            new_int_write_loc_list.push(new_loc);
            if let Some(v) = var_map.get_mut(&var_name.to_string()) {
                v.push((
                    LocationInfo::IntLocList(new_int_write_loc_list.clone()),
                    pc_range,
                ));
            } else {
                var_map.insert(
                    var_name.to_string(),
                    vec![(
                        LocationInfo::IntLocList(new_int_write_loc_list.clone()),
                        pc_range,
                    )],
                );
            }
        } else {
            println!("got into new_loc_list..");
            new_loc_list.push(new_loc);
            if let Some(v) = var_map.get_mut(&var_name.to_string()) {
                println!("push into new_loc_list..");
                v.push((LocationInfo::IntLocList(new_loc_list.clone()), pc_range));
            } else {
                println!("insert into new_loc_list..");
                var_map.insert(
                    var_name.to_string(),
                    vec![(LocationInfo::IntLocList(new_loc_list.clone()), pc_range)],
                );
            }
        }

        let locations = &mut unit.locations;
        let new_loc_list_id = if const_value_attr {
            locations.add(write::LocationList(new_write_loc_list))
        } else if read_location {
            locations.add(write::LocationList(new_read_loc_list))
        } else if int_loc_list {
            //println!("intermediate loc list : {:?}", new_int_write_loc_list);
            locations.add(write::LocationList(new_int_write_loc_list))
        } else {
            locations.add(write::LocationList(new_loc_list))
        };
        let attr_val = write::AttributeValue::LocationListRef(new_loc_list_id);
        let var_loc = get_var_loc(unit, &var.unwrap());
        if var_loc == None {
            //println!("Variable: {} has no location attribute available! Skipping for now..(TODO: Create a new attribute)", var_name);
            println!("[LOG]: Trying to add a new attribute - DW_AT_location..");
            let var_die = get_die(unit, &var.unwrap());
            var_die.set(DW_AT_location, attr_val);
            println!(
                "[LOG]: New Variable: {} location attribute added successfully!",
                var_name
            );
        } else {
            let var_loc = var_loc.unwrap();
            var_loc.set(attr_val);
            println!(
                "[LOG]: Variable: {} location attribute changed successfully!",
                var_name
            );
        }
        /* Rust tricks - changing the ordering - Moved the below block after above get_die()
         * block */
        {
            if const_value_attr {
                let var_die = get_die(unit, &var.unwrap());
                var_die.delete(DW_AT_const_value);
            }
            if let Some(true) = var_empty_scope.get(&var_name.to_string()) {
                let parent_die = get_die(unit, &parent.unwrap());
                if parent_die.tag().static_string() == Some("DW_TAG_lexical_block") {
                    parent_die.delete(gimli::DW_AT_ranges);
                }
            }
        }
    }

    // TODO: only add relocations for relocatable files
    let mut sections = write::Sections::new(WriterRelocate::new(EndianVec::new(LittleEndian)));
    dwarf.write(&mut sections).unwrap();
    let mut section_symbols = HashMap::new();

    let _: Result<(), gimli::Error> = sections.for_each_mut(|id, w| {
        define(
            id,
            out_object,
            &mut section_symbols,
            symbols,
            w.writer.take(),
            &w.relocations,
        );
        Ok(())
    });

    /*
    let frame = write::FrameTable::from(&eh_frame, &convert_address).unwrap();
    let mut out_eh_frame = write::EhFrame(WriterRelocate::new(EndianVec::new(LittleEndian)));
    frame.write_eh_frame(&mut out_eh_frame).unwrap();
    define(
        gimli::SectionId::EhFrame,
        out_object,
        &mut section_symbols,
        symbols,
        out_eh_frame.0.writer.take(),
        &out_eh_frame.0.relocations,
    );
    */
}

fn define(
    id: gimli::SectionId,
    out_object: &mut object_write::Object,
    section_symbols: &mut HashMap<gimli::SectionId, object_write::SymbolId>,
    symbols: &HashMap<SymbolIndex, object_write::SymbolId>,
    data: Vec<u8>,
    relocations: &[Relocation],
) {
    if data.is_empty() {
        return;
    }

    let section_id = out_object.add_section(
        vec![],
        id.name().as_bytes().to_vec(),
        object::SectionKind::Other,
    );
    let section = out_object.section_mut(section_id);
    section.set_data(data, 1);
    let symbol_id = out_object.section_symbol(section_id);
    section_symbols.insert(id, symbol_id);
    for relocation in link(section_symbols, symbols, relocations) {
        out_object.add_relocation(section_id, relocation).unwrap();
    }
}

fn link(
    section_symbols: &HashMap<gimli::SectionId, object_write::SymbolId>,
    symbols: &HashMap<SymbolIndex, object_write::SymbolId>,
    relocations: &[Relocation],
) -> Vec<object_write::Relocation> {
    let mut out_relocations = Vec::new();
    for reloc in relocations {
        match *reloc {
            Relocation::Section {
                offset,
                section,
                addend,
                size,
            } => {
                let symbol = match section_symbols.get(&section) {
                    Some(s) => *s,
                    None => {
                        eprintln!("Missing section {}", section.name());
                        continue;
                    }
                };
                out_relocations.push(object_write::Relocation {
                    offset,
                    size: size * 8,
                    kind: object::RelocationKind::Absolute,
                    encoding: object::RelocationEncoding::Generic,
                    symbol,
                    addend: addend as i64,
                });
            }
            Relocation::Symbol {
                offset,
                symbol,
                addend,
                kind,
                size,
            } => {
                let symbol = *symbols.get(&symbol).unwrap();
                out_relocations.push(object_write::Relocation {
                    offset,
                    size: size * 8,
                    kind,
                    encoding: object::RelocationEncoding::Generic,
                    symbol,
                    addend: addend as i64,
                });
            }
        }
    }
    out_relocations
}

pub fn is_rewrite_dwarf_section(section: &object::Section<'_, '_>) -> bool {
    if let Ok(name) = section.name() {
        if name.starts_with(".debug_") {
            match name {
                ".debug_aranges" | ".debug_abbrev" | ".debug_addr" | ".debug_info"
                | ".debug_line" | ".debug_line_str" | ".debug_loc" | ".debug_loclists"
                | ".debug_pubnames" | ".debug_pubtypes" | ".debug_ranges" | ".debug_rnglists"
                | ".debug_str" | ".debug_str_offsets" => {
                    return true;
                }
                _ => return false,
            }
        }
        /*
        if name == ".eh_frame" {
            return true;
        }
        */
    }
    false
}

type ReadRelocationMap = HashMap<usize, object::Relocation>;

fn get_section<'data>(
    file: &object::File<'data>,
    name: &str,
) -> (Cow<'data, [u8]>, ReadRelocationMap) {
    let mut relocations = ReadRelocationMap::default();
    let section = match file.section_by_name(name) {
        Some(section) => section,
        None => return (Cow::Borrowed(&[][..]), relocations),
    };
    for (offset64, mut relocation) in section.relocations() {
        let offset = offset64 as usize;
        if offset as u64 != offset64 {
            continue;
        }
        let offset = offset as usize;
        match relocation.kind() {
            object::RelocationKind::Absolute | object::RelocationKind::Relative => {
                match relocation.target() {
                    object::RelocationTarget::Symbol(symbol) => {
                        if let Ok(symbol) = file.symbol_by_index(symbol) {
                            let addend = symbol.address().wrapping_add(relocation.addend() as u64);
                            relocation.set_addend(addend as i64);
                            println!("Adding reloc {} {:?}", offset, relocation);
                            if relocations.insert(offset, relocation).is_some() {
                                println!(
                                    "Multiple relocations for section {} at offset 0x{:08x}",
                                    section.name().unwrap(),
                                    offset
                                );
                            }
                        } else {
                            println!(
                                "Relocation with invalid symbol for section {} at offset 0x{:08x}",
                                section.name().unwrap(),
                                offset
                            );
                        }
                    }
                    _ => {
                        println!(
                            "Unsupported relocation target for section {} at offset 0x{:08x}",
                            section.name().unwrap(),
                            offset
                        );
                    }
                }
            }
            _ => {
                println!(
                    "Unsupported relocation kind for section {} at offset 0x{:08x}",
                    section.name().unwrap(),
                    offset
                );
            }
        }
    }

    let data = section.uncompressed_data().unwrap();
    (data, relocations)
}

// gimli::read::Reader::read_address() returns u64, but gimli::write data structures wants
// a gimli::write::Address. To work around this, every time we read an address we add
// an Address to this map, and return that index in read_address(). Then later we
// convert that index back into the Address.
// Note that addresses 0 and !0 can have special meaning in DWARF (eg for range lists).
// 0 can also be appear as a default value for DW_AT_low_pc.
#[derive(Debug, Default)]
struct ReadAddressMap {
    addresses: RefCell<Vec<Address>>,
}

impl ReadAddressMap {
    fn add(&self, address: Address) -> usize {
        if address == Address::Constant(0) {
            // Must be zero because this may not be an address.
            return 0;
        }
        let mut addresses = self.addresses.borrow_mut();
        addresses.push(address);
        // Non-zero
        addresses.len()
    }

    fn get(&self, index: usize) -> Address {
        if index == 0 {
            Address::Constant(0)
        } else {
            let addresses = self.addresses.borrow();
            addresses[index - 1]
        }
    }
}

#[derive(Debug, Clone)]
struct ReaderRelocate<'a, R: read::Reader<Offset = usize>> {
    relocations: &'a ReadRelocationMap,
    addresses: &'a ReadAddressMap,
    section: R,
    reader: R,
}

impl<'a, R: read::Reader<Offset = usize>> ReaderRelocate<'a, R> {
    fn relocate(&self, offset: usize, value: u64) -> u64 {
        if let Some(relocation) = self.relocations.get(&offset) {
            match relocation.kind() {
                object::RelocationKind::Absolute => {
                    if relocation.has_implicit_addend() {
                        // Use the explicit addend too, because it may have the symbol value.
                        return value.wrapping_add(relocation.addend() as u64);
                    } else {
                        return relocation.addend() as u64;
                    }
                }
                _ => {}
            }
        }
        value
    }

    fn relocate_address(&self, offset: usize, value: u64) -> Option<Address> {
        if let Some(relocation) = self.relocations.get(&offset) {
            let symbol = match relocation.target() {
                object::RelocationTarget::Symbol(symbol) => symbol.0,
                _ => unimplemented!(),
            };
            let addend = match relocation.kind() {
                object::RelocationKind::Absolute | object::RelocationKind::Relative => {
                    if relocation.has_implicit_addend() {
                        // Use the explicit addend too, because it may have the symbol value.
                        value.wrapping_add(relocation.addend() as u64) as i64
                    } else {
                        relocation.addend()
                    }
                }
                _ => unimplemented!(),
            };
            Some(Address::Symbol { symbol, addend })
        } else {
            None
        }
    }
}

impl<'a, R: read::Reader<Offset = usize>> read::Reader for ReaderRelocate<'a, R> {
    type Endian = R::Endian;
    type Offset = R::Offset;

    fn read_address(&mut self, address_size: u8) -> read::Result<u64> {
        let offset = self.reader.offset_from(&self.section);
        let value = self.reader.read_address(address_size)?;
        //println!("read_address {} {:x}", offset, value);
        let address = ReaderRelocate::relocate_address(self, offset, value)
            .unwrap_or(Address::Constant(value));
        let addend = get_addr(address);
        if value != addend {
            eprintln!("value: {}, addend: {}", value, addend);
        }
        //println!("relocate_address {} {:?}", offset, address);
        let ret = self.addresses.add(address) as u64;
        if value == 0x4b6 || ret == 64 {
            eprintln!("value: {:x}, index: {}", value, ret);
        }
        //println!("index: {}", ret);
        Ok(ret)
    }

    fn read_offset(&mut self, format: gimli::Format) -> read::Result<usize> {
        let offset = self.reader.offset_from(&self.section);
        let value = self.reader.read_offset(format)?;
        //println!("read_offset {} {}", offset, value);
        <usize as read::ReaderOffset>::from_u64(self.relocate(offset, value as u64))
    }

    fn read_sized_offset(&mut self, size: u8) -> read::Result<usize> {
        let offset = self.reader.offset_from(&self.section);
        let value = self.reader.read_sized_offset(size)?;
        //println!("read_sized_offset {} {}", offset, value);
        <usize as read::ReaderOffset>::from_u64(self.relocate(offset, value as u64))
    }

    #[inline]
    fn split(&mut self, len: Self::Offset) -> read::Result<Self> {
        let mut other = self.clone();
        other.reader.truncate(len)?;
        self.reader.skip(len)?;
        Ok(other)
    }

    // All remaining methods simply delegate to `self.reader`.

    #[inline]
    fn endian(&self) -> Self::Endian {
        self.reader.endian()
    }

    #[inline]
    fn len(&self) -> Self::Offset {
        self.reader.len()
    }

    #[inline]
    fn empty(&mut self) {
        self.reader.empty()
    }

    #[inline]
    fn truncate(&mut self, len: Self::Offset) -> read::Result<()> {
        self.reader.truncate(len)
    }

    #[inline]
    fn offset_from(&self, base: &Self) -> Self::Offset {
        self.reader.offset_from(&base.reader)
    }

    #[inline]
    fn offset_id(&self) -> gimli::ReaderOffsetId {
        self.reader.offset_id()
    }

    #[inline]
    fn lookup_offset_id(&self, id: gimli::ReaderOffsetId) -> Option<Self::Offset> {
        self.reader.lookup_offset_id(id)
    }

    #[inline]
    fn find(&self, byte: u8) -> read::Result<Self::Offset> {
        self.reader.find(byte)
    }

    #[inline]
    fn skip(&mut self, len: Self::Offset) -> read::Result<()> {
        self.reader.skip(len)
    }

    #[inline]
    fn to_slice(&self) -> read::Result<Cow<'_, [u8]>> {
        self.reader.to_slice()
    }

    #[inline]
    fn to_string(&self) -> read::Result<Cow<'_, str>> {
        self.reader.to_string()
    }

    #[inline]
    fn to_string_lossy(&self) -> read::Result<Cow<'_, str>> {
        self.reader.to_string_lossy()
    }

    #[inline]
    fn read_slice(&mut self, buf: &mut [u8]) -> read::Result<()> {
        self.reader.read_slice(buf)
    }
}

#[derive(Debug, Clone)]
pub enum Relocation {
    Section {
        offset: u64,
        section: gimli::SectionId,
        addend: i32,
        size: u8,
    },
    Symbol {
        offset: u64,
        symbol: SymbolIndex,
        addend: i32,
        kind: object::RelocationKind,
        size: u8,
    },
}

#[derive(Debug, Clone)]
struct WriterRelocate<W: write::Writer> {
    relocations: Vec<Relocation>,
    writer: W,
}

impl<W: write::Writer> WriterRelocate<W> {
    fn new(writer: W) -> Self {
        WriterRelocate {
            relocations: Vec::new(),
            writer,
        }
    }
}

impl<W: write::Writer> write::Writer for WriterRelocate<W> {
    type Endian = W::Endian;

    fn endian(&self) -> Self::Endian {
        self.writer.endian()
    }

    fn len(&self) -> usize {
        self.writer.len()
    }

    fn write(&mut self, bytes: &[u8]) -> write::Result<()> {
        self.writer.write(bytes)
    }

    fn write_at(&mut self, offset: usize, bytes: &[u8]) -> write::Result<()> {
        self.writer.write_at(offset, bytes)
    }

    fn write_address(&mut self, address: Address, size: u8) -> write::Result<()> {
        match address {
            Address::Constant(val) => self.write_udata(val, size),
            Address::Symbol { symbol, addend } => {
                let offset = self.len() as u64;
                self.relocations.push(Relocation::Symbol {
                    offset,
                    symbol: SymbolIndex(symbol),
                    addend: addend as i32,
                    kind: object::RelocationKind::Absolute,
                    size,
                });
                self.write_udata(0, size)
            }
        }
    }

    fn write_eh_pointer(
        &mut self,
        address: Address,
        eh_pe: gimli::DwEhPe,
        _size: u8,
    ) -> write::Result<()> {
        println!("write_eh_pointer {} {:?}", self.len(), address);
        match (address, eh_pe.application(), eh_pe.format()) {
            (Address::Constant(value), gimli::DW_EH_PE_absptr, gimli::DW_EH_PE_sdata4) => {
                self.write_u32(value as u32)
            }
            (Address::Symbol { symbol, addend }, gimli::DW_EH_PE_pcrel, gimli::DW_EH_PE_sdata4) => {
                let offset = self.len() as u64;
                self.relocations.push(Relocation::Symbol {
                    offset,
                    symbol: SymbolIndex(symbol),
                    addend: addend as i32,
                    kind: object::RelocationKind::Relative,
                    size: 4,
                });
                self.write_u32(0)
            }
            _ => unimplemented!("{:?} {:?}", address, eh_pe),
        }
    }

    fn write_offset(
        &mut self,
        val: usize,
        section: gimli::SectionId,
        size: u8,
    ) -> write::Result<()> {
        let offset = self.len() as u64;
        self.relocations.push(Relocation::Section {
            offset,
            section,
            addend: val as i32,
            size,
        });
        self.write_udata(0, size)
    }

    fn write_offset_at(
        &mut self,
        offset: usize,
        val: usize,
        section: gimli::SectionId,
        size: u8,
    ) -> write::Result<()> {
        self.relocations.push(Relocation::Section {
            offset: offset as u64,
            section,
            addend: val as i32,
            size,
        });
        self.write_udata_at(offset, 0, size)
    }
}
