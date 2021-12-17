use std::borrow::Cow;

use gimli::read::EndianSlice;
use gimli::read::Reader;
use gimli::{self, read, LittleEndian};
use object::{self, Object, ObjectSection};
use std::str;

extern crate capstone;
use capstone::prelude::*;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::ops::Bound::Excluded;
use std::ops::Bound::Included;

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
                                //eprintln!("function {} FOUND!", func_name);
                                flag = true;
                                //break;
                            }
                        } else if let read::AttributeValue::String(reader) = attr.value() {
                            let s = reader.to_string().unwrap();
                            if s.to_string() == func_name {
                                flag = true;
                            }
                        } else {
                            eprintln!("read::AttributeValue of this type not handled yet!\n");
                        }
                    } else if attr.name().static_string().unwrap() == "DW_AT_low_pc" {
                        if let read::AttributeValue::Addr(start_addr) = attr.value() {
                            func_start_addr = Some(start_addr);
                            //eprintln!("func start addr: {:x}", start_addr);
                        }
                    } else if attr.name().static_string().unwrap() == "DW_AT_high_pc" {
                        if let read::AttributeValue::Udata(end_offset) = attr.value() {
                            func_end_offset = Some(end_offset);
                            //eprintln!("func end offset: {}", end_offset);
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

pub fn read_dwarf(
    file: &object::File<'_>,
    func_name: &str,
    insn_map_str: &str,
) -> (
    HashMap<String, BTreeSet<(u64, u64, bool)>>,
    BTreeSet<u64>,
    i64,
) {
    fn get_reader<'a>(
        data: &'a [u8],
        relocations: &'a ReadRelocationMap,
    ) -> ReaderRelocate<'a, EndianSlice<'a, LittleEndian>> {
        let section = EndianSlice::new(data, LittleEndian);
        let reader = section.clone();
        ReaderRelocate {
            relocations,
            section,
            reader,
        }
    };

    let no_section = (Cow::Borrowed(&[][..]), ReadRelocationMap::default());

    let (debug_abbrev_data, debug_abbrev_relocs) = get_section(file, ".debug_abbrev");
    let (debug_addr_data, debug_addr_relocs) = get_section(file, ".debug_addr");
    let (debug_info_data, debug_info_relocs) = get_section(file, ".debug_info");
    let (debug_line_data, debug_line_relocs) = get_section(file, ".debug_line");
    let (debug_line_str_data, debug_line_str_relocs) = get_section(file, ".debug_line_str");
    let (debug_loc_data, debug_loc_relocs) = get_section(file, ".debug_loc");
    let (debug_loclists_data, debug_loclists_relocs) = get_section(file, ".debug_loclists");
    let (debug_ranges_data, debug_ranges_relocs) = get_section(file, ".debug_ranges");
    /*let (debug_aranges_data, debug_aranges_relocs) = get_section(file, ".debug_aranges");*/
    let (debug_rnglists_data, debug_rnglists_relocs) = get_section(file, ".debug_rnglists");
    let (debug_str_data, debug_str_relocs) = get_section(file, ".debug_str");
    let (debug_str_offsets_data, debug_str_offsets_relocs) =
        get_section(file, ".debug_str_offsets");
    let (debug_types_data, debug_types_relocs) = get_section(file, ".debug_types");
    let debug_addr = read::DebugAddr::from(get_reader(&debug_addr_data, &debug_addr_relocs));
    let dwarf = read::Dwarf {
        debug_abbrev: read::DebugAbbrev::from(get_reader(&debug_abbrev_data, &debug_abbrev_relocs)),
        debug_addr: debug_addr,
        debug_info: read::DebugInfo::from(get_reader(&debug_info_data, &debug_info_relocs)),
        debug_line: read::DebugLine::from(get_reader(&debug_line_data, &debug_line_relocs)),
        debug_line_str: read::DebugLineStr::from(get_reader(
            &debug_line_str_data,
            &debug_line_str_relocs,
        )),
        debug_str: read::DebugStr::from(get_reader(&debug_str_data, &debug_str_relocs)),
        debug_str_offsets: read::DebugStrOffsets::from(get_reader(
            &debug_str_offsets_data,
            &debug_str_offsets_relocs,
        )),
        debug_str_sup: read::DebugStr::from(get_reader(&no_section.0, &no_section.1)),
        debug_types: read::DebugTypes::from(get_reader(&debug_types_data, &debug_types_relocs)),
        locations: read::LocationLists::new(
            read::DebugLoc::from(get_reader(&debug_loc_data, &debug_loc_relocs)),
            read::DebugLocLists::from(get_reader(&debug_loclists_data, &debug_loclists_relocs)),
        ),
        ranges: read::RangeLists::new(
            read::DebugRanges::from(get_reader(&debug_ranges_data, &debug_ranges_relocs)),
            read::DebugRngLists::from(get_reader(&debug_rnglists_data, &debug_rnglists_relocs)),
        ),
    };

    let (func_entry_offset, func_start_addr, func_end_offset) =
        get_func_entry_offset(&dwarf, func_name);
    let func_start_addr = func_start_addr.unwrap();
    let func_end_offset = func_end_offset.unwrap();
    let func_end_addr = func_start_addr + func_end_offset;

    /* Instructions Disassembly using Capstone */
    let mut text_data = None;
    if let Some(section) = file.section_by_name(".text") {
        text_data = Some(section.uncompressed_data().unwrap());
    }
    let cs = Capstone::new()
        .x86()
        .mode(arch::x86::ArchMode::Mode32)
        .syntax(arch::x86::ArchSyntax::Att)
        .detail(true)
        .build()
        .expect("Failed to create capstone object");
    let insns = cs.disasm_all(&text_data.unwrap(), 0x0000).unwrap();
    /*let insns = cs.disasm_count(&text_data.unwrap(), 0x3e00, func_end_offset.try_into().unwrap())
    .expect("Failed to disassemble");*/
    //eprintln!("Found {} instructions", insns.len());

    let mut insn_map = HashMap::new();
    let mut insn_set = BTreeSet::new();

    let mut lines = insn_map_str/*.as_str()*/.lines();
    let line = lines.next().unwrap();
    assert!(line == "=insn_pcs", "Invalid insn-map-file format!");
    for line in lines {
        if line == "=End" {
            break;
        }
        let index_and_pc: Vec<&str> = line.trim().split(':').collect();
        let pc_str = index_and_pc[1];
        let pc = pc_str.trim().trim_start_matches("0x");
        eprintln!("pc: {}", pc);
        let pc = u64::from_str_radix(pc, 16).unwrap();
        if pc != 0x7fffffff {
            insn_map.insert(pc, 0);
            insn_set.insert(pc);
        }
    }


    let mut insn_size = HashMap::new();
    for i in insns.iter() {
        ////eprintln!("{:x}", i.address());
        if i.address() >= func_start_addr && i.address() < func_end_addr {
            //eprintln!("{}", i);
            /* Not using insn_map and insn_set here now */
            //insn_map.insert(i.address(), 0);
            //insn_set.insert(i.address());
            insn_size.insert(i.address(), i.bytes().len());
        }
    }


    let mut results_map = HashMap::new();

    let units = &mut dwarf.units();
    let unit_header = units.next().unwrap();
    let unit = dwarf.unit(unit_header.unwrap()).unwrap();
    let encoding = unit.encoding();

    let mut entries = unit.entries_at_offset(func_entry_offset.unwrap()).unwrap();

    let mut depth = 0;
    let mut first = true;
    let mut low_pc = None;
    let mut high_pc = None;
    let mut scope_ranges: Option<Vec<(u64, u64)>> = None;
    while let Some((index, entry)) = entries.next_dfs().unwrap() {
        let mut var_name = None;
        let mut var_info = None;
        depth += index;
        if !first && depth <= 0 {
            break;
        }
        if first == true {
            first = false;
        }
        //eprintln!("Entry tag: {:?}", entry.tag().static_string());
        //eprintln!("Index : {}", index);
        if true {
            let mut attrs = entry.attrs();
            let mut low_pc_attr_present = false;
            let mut high_pc_attr_present = false;
            let mut ranges_attr_present = false;
            while let Some(attr) = attrs.next().unwrap() {
                if attr.name().static_string().unwrap() == "DW_AT_name" {
                    if let read::AttributeValue::DebugStrRef(debug_str_offset) = attr.value() {
                        let str_val = dwarf.string(debug_str_offset).unwrap();
                        var_name = Some(str_val.to_string().unwrap().into_owned());
                    //eprintln!("Variable name: {:?}", str_val.to_string().unwrap());
                    //eprintln!("Name: {:?}", str_val.to_string().unwrap());
                    } else if let read::AttributeValue::String(reader) = attr.value() {
                        let s = reader.to_string().unwrap();
                        var_name = Some(s.to_string());
                    } else {
                        eprintln!("AttributeValue of this type is not handled yet!\n");
                    }
                //eprintln!("Variable Name: {:?}", attr.value());
                } else if attr.name().static_string().unwrap() == "DW_AT_location" {
                    if let read::AttributeValue::LocationListsRef(location_lists_offset) =
                        attr.value()
                    {
                        let mut loclist_iter =
                            dwarf.locations(&unit, location_lists_offset).unwrap();
                        let mut locations = BTreeSet::new();
                        while let Some(loclist_entry) = loclist_iter.next().unwrap() {
                            //eprintln!("loc range: {:?}", loclist_entry.range);
                            //let mut write_expr = gimli::write::Expression::from(loclist_entry.data.clone(), encoding, None, None, None, &convert_address).unwrap();
                            let mut ops_iter = loclist_entry.data.operations(encoding);
                            let mut is_const = false;
                            if let Some(op) = ops_iter.next().unwrap() {
                                if let gimli::read::Operation::SignedConstant { value: _ } =
                                    op.clone()
                                {
                                    is_const = true;
                                } else if let gimli::read::Operation::UnsignedConstant {
                                    value: _,
                                } = op
                                {
                                    is_const = true;
                                }
                            }
                            if let Some(op) = ops_iter.next().unwrap() {
                                if let gimli::read::Operation::StackValue = op {
                                    is_const = is_const && true;
                                } else {
                                    is_const = false;
                                }
                            }

                            /*locations.insert((
                                loclist_entry.range.begin,
                                loclist_entry.range.end,
                                is_const,
                            ));*/
                            if scope_ranges != None {
                                let mut contiguous_range_so_far = Vec::new();
                                //let mut tmp = 0;
                                for &pc in insn_set.range((
                                    Included(&loclist_entry.range.begin),
                                    Excluded(&loclist_entry.range.end),
                                )) {
                                    let mut count_this_pc = false;
                                    for &(begin, end) in scope_ranges.clone().unwrap().iter() {
                                        if pc >= begin && pc < end {
                                            count_this_pc = true;
                                            break;
                                        }
                                    }
                                    if count_this_pc {
                                        contiguous_range_so_far.push(pc);
                                        //eprintln!("{}", pc);
                                        let pc_cnt = insn_map.get_mut(&pc).unwrap();
                                        *pc_cnt = *pc_cnt + 1;
                                    //tmp += 1;
                                    } else {
                                        if !contiguous_range_so_far.is_empty() {
                                            let range_begin = contiguous_range_so_far[0];
                                            let range_end = contiguous_range_so_far
                                                [contiguous_range_so_far.len() - 1];
                                            let mut r = insn_set.range(range_end..);
                                            r.next();
                                            let range_end = match r.next() {
                                                None => range_end + insn_size[&range_end] as u64, /* beware -- type casting */
                                                next_pc => *next_pc.unwrap(),
                                            };
                                            locations.insert((range_begin, range_end, is_const));
                                            contiguous_range_so_far.clear();
                                        }
                                    }
                                }
                                if !contiguous_range_so_far.is_empty() {
                                    let range_begin = contiguous_range_so_far[0];
                                    let range_end =
                                        contiguous_range_so_far[contiguous_range_so_far.len() - 1];
                                    let mut r = insn_set.range(range_end..);
                                    r.next();
                                    let range_end = match r.next() {
                                        None => range_end + insn_size[&range_end] as u64, /* beware -- type casting */
                                        next_pc => *next_pc.unwrap(),
                                    };
                                    locations.insert((range_begin, range_end, is_const));
                                    contiguous_range_so_far.clear();
                                }
                            } else {
                                eprintln!("WARNING: No scope defined for current variable!");
                            }
                            /*println!(
                                "actual++: {} | {}->{}",
                                tmp, loclist_entry.range.begin, loclist_entry.range.end
                            );*/
                        }
                        var_info = Some(locations);
                    } else if scope_ranges != None {
                        if let read::AttributeValue::Exprloc(dwarf_expr) = attr.value() {
                            let mut locations = BTreeSet::new();
                            //let mut write_expr = gimli::write::Expression::from(dwarf_expr.clone(), encoding, None, None, None, &convert_address).unwrap();
                            let mut ops_iter = dwarf_expr.operations(encoding);
                            let mut is_const = false;
                            if let Some(op) = ops_iter.next().unwrap() {
                                if let gimli::read::Operation::SignedConstant { value: _ } =
                                    op.clone()
                                {
                                    is_const = true;
                                } else if let gimli::read::Operation::UnsignedConstant {
                                    value: _,
                                } = op
                                {
                                    is_const = true;
                                }
                            }
                            if let Some(op) = ops_iter.next().unwrap() {
                                if let gimli::read::Operation::StackValue = op {
                                    is_const = is_const && true;
                                } else {
                                    is_const = false;
                                }
                            }
                            for &(begin, end) in scope_ranges.clone().unwrap().iter() {
                                locations.insert((begin, end, is_const));
                                //let mut tmp = 0;
                                for &pc in insn_set.range((Included(&begin), Excluded(&end))) {
                                    //eprintln!("{}", pc);
                                    let pc_cnt = insn_map.get_mut(&pc).unwrap();
                                    *pc_cnt = *pc_cnt + 1;
                                    //tmp += 1;
                                }
                            }
                            var_info = Some(locations);

                        //println!("actual++: {} | {}->{}", tmp, low_pc, high_pc);
                        } else {
                            eprintln!("read::AttributeValue -- location -- not handled yet!\n");
                        }
                    } else {
                        eprintln!(
                            "WARNING: Possibly because no scope defined for current variable!?"
                        );
                    }
                } else if attr.name().static_string().unwrap() == "DW_AT_const_value" {
                    if scope_ranges != None {
                        let mut error = false;
                        let _data = match attr.value() {
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
                        let mut locations = BTreeSet::new();
                        /*let mut write_expr = gimli::write::Expression::new();
                        write_expr.op_consts(data as i64);
                        write_expr.op(DW_OP_stack_value);*/
                        for &(begin, end) in scope_ranges.clone().unwrap().iter() {
                            locations.insert((begin, end, /*is_const*/ true));
                            //let mut tmp = 0;
                            for &pc in insn_set.range((Included(&begin), Excluded(&end))) {
                                //eprintln!("{}", pc);
                                let pc_cnt = insn_map.get_mut(&pc).unwrap();
                                *pc_cnt = *pc_cnt + 1;
                                //tmp += 1;
                            }
                        }
                        var_info = Some(locations);
                    //println!("actual++: {} | {}->{}", tmp, low_pc, high_pc);
                    } else {
                        eprintln!("WARNING: No scope defined for current variable!");
                    }
                } else if attr.name().static_string().unwrap() == "DW_AT_low_pc" {
                    if let read::AttributeValue::Addr(_addr) = attr.value() {
                        low_pc_attr_present = true;
                        /*low_pc = Some(addr);*/
                    }
                } else if attr.name().static_string().unwrap() == "DW_AT_high_pc" {
                    if let read::AttributeValue::Udata(_offset) = attr.value() {
                        high_pc_attr_present = true;
                        /*high_pc = Some(offset);*/
                    }
                } else if attr.name().static_string().unwrap() == "DW_AT_ranges" {
                    if let read::AttributeValue::RangeListsRef(_offset) = attr.value() {
                        ranges_attr_present = true;
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
                }
                if let read::AttributeValue::Udata(offset) = high_pc_val {
                    high_pc = Some(offset);
                }
                if low_pc != None && high_pc != None {
                    let low_pc = low_pc.unwrap();
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
                        scope_vec.push((range_entry.begin, range_entry.end));
                    }
                    if scope_vec.is_empty() {
                        scope_ranges = None;
                    } else {
                        scope_ranges = Some(scope_vec);
                    }
                }
            }

            if entry.tag() == gimli::DW_TAG_formal_parameter
                || entry.tag() == gimli::DW_TAG_variable
            {
                if var_name != None && var_info != None {
                    results_map.insert(
                        var_name.clone().unwrap().clone(),
                        var_info.clone().unwrap().clone(),
                    );
                }
            }
        }
    }
    //eprintln!("While loop exited!");
    //eprintln!("insn_map = {:?}", insn_map);
    //eprintln!("insn_set = {:?}", insn_set);
    let mut summation: i64 = 0;
    //println!("=actual_coverage");
    for key in &insn_set {
        let val = insn_map.get(&key).unwrap();
        //println!("{:x}: {}", key, val);
        summation += val;
    }
    let cumulative_actual_count: i64 = summation;
    //println!("=actual_total\n{}", summation);
    (results_map, insn_set, cumulative_actual_count)
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
                            //println!("Adding reloc {} {:?}", offset, relocation);
                            if relocations.insert(offset, relocation).is_some() {
                                /*println!(
                                    "Multiple relocations for section {} at offset 0x{:08x}",
                                    section.name().unwrap(),
                                    offset
                                );*/
                            }
                        } else {
                            /*println!(
                                "Relocation with invalid symbol for section {} at offset 0x{:08x}",
                                section.name().unwrap(),
                                offset
                            );*/
                        }
                    }
                    _ => {
                        /*println!(
                            "Unsupported relocation target for section {} at offset 0x{:08x}",
                            section.name().unwrap(),
                            offset
                        );*/
                    }
                }
            }
            _ => {
                /*println!(
                    "Unsupported relocation kind for section {} at offset 0x{:08x}",
                    section.name().unwrap(),
                    offset
                );*/
            }
        }
    }

    let data = section.uncompressed_data().unwrap();
    (data, relocations)
}

#[derive(Debug, Clone)]
struct ReaderRelocate<'a, R: read::Reader<Offset = usize>> {
    relocations: &'a ReadRelocationMap,
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
}

impl<'a, R: read::Reader<Offset = usize>> read::Reader for ReaderRelocate<'a, R> {
    type Endian = R::Endian;
    type Offset = R::Offset;

    fn read_address(&mut self, address_size: u8) -> read::Result<u64> {
        let offset = self.reader.offset_from(&self.section);
        let value = self.reader.read_address(address_size)?;
        Ok(self.relocate(offset, value))
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
