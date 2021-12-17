use env_logger;
use memmap;
use std::collections::BTreeSet;
use std::{env, fs, process};

mod dwarf;
use dwarf::*;

use std::ops::Bound::{Excluded, Included};
use std::convert::TryInto;
use std::cmp;

fn get_print_value(value: usize) -> String {
    if value > 0 {
        value.to_string()
    } else {
        String::from("-")
    }
}

fn main() {
    env_logger::init();
    let mut args = env::args();
    let arglen = args.len();
    if arglen < 5 {
        eprintln!(
            "Usage: {} <before_obj_file> <after_obj_file> <func_name> <insn_map_file>",
            args.next().unwrap()
        );
        process::exit(1);
    }
    args.next();
    let before_obj_file_path = args.next().unwrap();
    let after_obj_file_path = args.next().unwrap();
    let func_name = args.next().unwrap();
    let insn_map_file = args.next().unwrap();
    let before_obj_file = match fs::File::open(&before_obj_file_path) {
        Ok(file) => file,
        Err(err) => {
            eprintln!("Failed to open file '{}': {}", before_obj_file_path, err);
            process::exit(1);
        }
    };
    let before_obj_file = match unsafe { memmap::Mmap::map(&before_obj_file) } {
        Ok(mmap) => mmap,
        Err(err) => {
            eprintln!("Failed to map file '{}': {}", before_obj_file_path, err);
            process::exit(1);
        }
    };
    let before_obj = match object::File::parse(&*before_obj_file) {
        Ok(obj) => obj,
        Err(err) => {
            eprintln!("Failed to parse file '{}': {}", before_obj_file_path, err);
            process::exit(1);
        }
    };
    let after_obj_file = match fs::File::open(&after_obj_file_path) {
        Ok(file) => file,
        Err(err) => {
            eprintln!("Failed to open file '{}': {}", after_obj_file_path, err);
            process::exit(1);
        }
    };
    let after_obj_file = match unsafe { memmap::Mmap::map(&after_obj_file) } {
        Ok(mmap) => mmap,
        Err(err) => {
            eprintln!("Failed to map file '{}': {}", after_obj_file_path, err);
            process::exit(1);
        }
    };
    let after_obj = match object::File::parse(&*after_obj_file) {
        Ok(obj) => obj,
        Err(err) => {
            eprintln!("Failed to parse file '{}': {}", after_obj_file_path, err);
            process::exit(1);
        }
    };
    let insn_map = fs::read_to_string(insn_map_file).expect("Failed to open insn_map_file");

    let (before_results_map, _insns_set, before_actual_count) = read_dwarf(&before_obj, &func_name, &insn_map);
    let (after_results_map, insns_set, after_actual_count) = read_dwarf(&after_obj, &func_name, &insn_map);
    //println!("before_results_map: {:?}\n", before_results_map);
    //println!("after_results_map: {:?}\n", after_results_map);
    //println!("insns_set: {:?}\n", insns_set);

    let mut improv_or_missing_pcs = BTreeSet::new(); // unique PCs count considering both Improved and Missing debug info updates
    let mut improv_or_missing_vars = BTreeSet::new(); // variables having either Improved or Missing debug info update
    let mut improv_or_missing_pc_var_pairs: i64 = 0; // no of pc-var pairs (cumulative counting of PCs considering both Improved and Missing updates)

    let mut const_to_non_const_count = 0;
    let mut improv_pcs = BTreeSet::new();
    let mut improv_var_cnt = 0;
    for (var_name, before_var_info) in &before_results_map {
        let mut count_var = false;
        let after_var_info = after_results_map.get(var_name);
        if after_var_info == None {
            continue;
        }
        let after_var_info = after_var_info.unwrap();
        for insn in &insns_set {
            let mut const_at_src = false;
            for (begin, end, is_const) in before_var_info {
                if insn >= begin && insn < end && *is_const == true {
                    const_at_src = true;
                    break;
                }
            }
            if const_at_src == true {
                for (begin, end, is_const) in after_var_info {
                    if insn >= begin && insn < end && *is_const == false {
                        const_to_non_const_count += 1;
                        improv_pcs.insert(insn);
                        improv_or_missing_pcs.insert(insn);
                        improv_or_missing_pc_var_pairs += 1;
                        count_var = true;
                        break;
                    }
                }
            }
        }
        if count_var {
            improv_or_missing_vars.insert(var_name);
            improv_var_cnt += 1;
        }
    }
    //println!("improved_pcs = {:?}", improv_pcs);
    let mut missing_pcs = BTreeSet::new();
    let mut missing_count = 0;
    let mut missing_var_cnt = 0;
    for (var_name, var_info) in &after_results_map {
        let mut count_var = false;
        if before_results_map.get(var_name) == None {
            for insn in &insns_set {
                for (begin, end, _is_const) in var_info {
                    if insn >= begin && insn < end {
                        missing_pcs.insert(insn);
                        improv_or_missing_pcs.insert(insn);
                        improv_or_missing_pc_var_pairs += 1;
                        missing_count += 1;
                        count_var = true;
                        break;
                    }
                }
            }
        } else {
            //println!("before_map has the variable {}!\n", var_name);
            let before_var_info = before_results_map.get(var_name).unwrap();
            for (begin, end, _) in var_info {
                for insn in insns_set.range((Included(begin), Excluded(end))) {
                    let mut found = false;
                    for (before_begin, before_end, _) in before_var_info {
                        if insn >= before_begin && insn < before_end {
                            found = true;
                            break;
                        }
                    }
                    if found == false {
                        missing_pcs.insert(insn);
                        improv_or_missing_pcs.insert(insn);
                        improv_or_missing_pc_var_pairs += 1;
                        missing_count += 1;
                        count_var = true;
                    }
                }
            }
        }
        if count_var {
            improv_or_missing_vars.insert(var_name);
            missing_var_cnt += 1;
        }
    }
    //println!("missing_pcs = {:?}", missing_pcs);
    //println!("const_to_non_const_count = {}\n", const_to_non_const_count);
    //println!("improved pcs = {}\n", improv_pcs.len());
    //println!("total count = {}\n", insns_set.len());
    //println!("Function name, Improved PCs, improved-var-count, Missing PCs, missing-var-count, Total PCs, Before Actual Count, After Actual Count");
    println!(
        "{}, {}/{}, {}, {}/{}/{}",
        func_name,
        insns_set.len(),
        //get_print_value(improv_or_missing_pcs.len()),
        improv_or_missing_pcs.len(),
        //get_print_value(improv_or_missing_vars.len()),
        improv_or_missing_vars.len(),
        before_actual_count,
        after_actual_count - before_actual_count,
        //get_print_value((improv_or_missing_pc_var_pairs - (after_actual_count - before_actual_count)).try_into().unwrap())
        cmp::max(0, improv_or_missing_pc_var_pairs - (cmp::max(0, after_actual_count - before_actual_count)))
    );
}
