use std::{process::exit, collections::HashMap, io::{self, Read}};

use crate::emulation_unit::UniversalMachine;

//UNIVERSAL MACHINE INSTRUCTION TYPE
type Umi = u32;

//struct to instructions and registers based on preset width and lsb values
pub struct Field {
    width:u32,
    lsb:u32
}

//list of Fields to dercribe certain Registes and OPCODES
static RA: Field = Field {width:3, lsb:6};
static RB: Field = Field {width:3, lsb:3};
static RC: Field = Field {width:3, lsb:0};
static RL: Field = Field {width:3, lsb:25};
static VL: Field = Field {width:25, lsb:0};
static OPCODE: Field = Field {width:4, lsb:28};


/// Public function to retrive the value at a given field value
/// 
/// # Arguments
/// * `field`: The specific feild you want to retrive a value at
/// * `instruction`: The codeword to derive the values from
pub fn get(field: &Field, instruction: Umi) -> u32 {
    (instruction >> field.lsb) & mask(field.width)
}

/// Private function to retrive the value at a given field value
/// 
/// # Arguments
/// * `bits`: the codeword
fn mask(bits:u32) -> u32 { (1 << bits) - 1}

//ENUM naming all the OPCODES
#[derive(Debug, PartialEq, Clone, Copy)]
enum Opcode {
    CMov,
    Load,
    Store,
    Add,
    Mul,
    Div,
    Nand,
    Halt,
    MapSegment,
    UnmapSegment,
    Output,
    Input,
    LoadProgram,
    LoadValue,
}

/// Private function to conditionally move two values stored in regeisters, in format if (r{} != 0) r{} := r{};", get(&RC, inst), get(&RA, inst), get(&RB, inst)
/// 
/// # Arguments
/// * `regs`: the registers in the UM
/// * `p_count`: the program counter
/// * `reg_a`: a register pointer
/// * `reg_b`: a register pointer
/// * `reg_c`: a register pointer
fn cmov(regs:&mut Vec<u32>, p_count: &mut usize, reg_a:u32, reg_b:u32, reg_c:u32){
  if regs[reg_c as usize] != 0{
    regs[reg_a as usize] = regs[reg_b as usize];
    *p_count += 1;
  }
  else {
    *p_count += 1;
  }
}

/// Private function to divide two values stored in regeisters, in format r{} := r{} + r{};", get(&RA, inst), get(&RB, inst), get(&RC, inst)
/// 
/// # Arguments
/// * `regs`: the registers in the UM
/// * `p_count`: the program counter
/// * `reg_a`: a register pointer
/// * `reg_b`: a register pointer
/// * `reg_c`: a register pointer
fn add(regs:&mut Vec<u32>, p_count: &mut usize, reg_a:u32, reg_b:u32, reg_c:u32){
  regs[reg_a as usize] = ((regs[reg_b as usize] as usize + regs[reg_c as usize] as usize) % usize::pow(2, 32))as u32;
  *p_count += 1;
}

/// Private function to divide two values stored in regeisters, in format r{} := r{} * r{};", get(&RA, inst), get(&RB, inst), get(&RC, inst)
/// 
/// # Arguments
/// * `regs`: the registers in the UM
/// * `p_count`: the program counter
/// * `reg_a`: a register pointer
/// * `reg_b`: a register pointer
/// * `reg_c`: a register pointer
fn multiply(regs:&mut Vec<u32>, p_count: &mut usize, reg_a:u32, reg_b:u32, reg_c:u32){
  regs[reg_a as usize] = ((regs[reg_b as usize] as usize * regs[reg_c as usize] as usize) % usize::pow(2, 32))as u32;
  *p_count += 1;
}

/// Private function to divide two values stored in regeisters, in format r{} := r{} / r{};", get(&RA, inst), get(&RB, inst), get(&RC, inst)
/// 
/// # Arguments
/// * `regs`: the registers in the UM
/// * `p_count`: the program counter
/// * `reg_a`: a register pointer
/// * `reg_b`: a register pointer
/// * `reg_c`: a register pointer
fn divide(regs:&mut Vec<u32>, p_count: &mut usize, reg_a:u32, reg_b:u32, reg_c:u32){
  //println!("{}", p_count);
  regs[reg_a as usize] = regs[reg_b as usize] / regs[reg_c as usize];
  *p_count += 1;
}

/// Private function to nand two values stored in regeisters, in format r{} := r{} nand r{};", get(&RA, inst), get(&RB, inst), get(&RC, inst)
/// 
/// # Arguments
/// * `regs`: the registers in the UM
/// * `p_count`: the program counter
/// * `reg_a`: a register pointer
/// * `reg_b`: a register pointer
/// * `reg_c`: a register pointer
fn nand(regs:&mut Vec<u32>, p_count: &mut usize, reg_a:u32, reg_b:u32, reg_c:u32){
  regs[reg_a as usize] = !(regs[reg_b as usize] & regs[reg_c as usize]);
  *p_count += 1;
}

/// Private function to load values directly to a regeister, in format r{} := {};", get(&RL, inst), get(&VL, inst)
/// 
/// # Arguments
/// * `regs`: the registers in the UM
/// * `p_count`: the program counter
/// * `reg_l`: a register pointer
fn load_value(regs:&mut Vec<u32>, p_count: &mut usize, reg_l:u32, val:u32) {
  regs[reg_l as usize] = val;
  *p_count += 1;
} 

///Private function to output values to stdout, in format output r{};", get(&RC, inst)
/// 
/// # Arguments
/// * `regs`: the registers in the UM
/// * `p_count`: the program counter
/// * `reg_c`: a register pointer
fn output(regs:&mut Vec<u32>, p_count: &mut usize, reg_c:u32) {
  let r = u8::try_from(regs[reg_c as usize]).unwrap();
  print!("{}", r as char);
  *p_count += 1;
}

//format!("r{} := m[r{}][r{}];", get(&RA, inst), get(&RB, inst), get(&RC, inst))
///Private function to load values into segemented memory, in format r{} := m[r{}][r{}];", get(&RA, inst), get(&RB, inst), get(&RC, inst)
/// 
/// # Arguments
/// * `regs`: the registers in the UM
/// * `seg`: the segmented memory in the UM
/// * `p_count`: the program counter
/// * `reg_a`: a register number to point to in the UM
/// * `reg_b`: a register number to point to in the UM
/// * `reg_c`: a register number to point to in the UM
fn load(regs:&mut Vec<u32>, seg: &mut HashMap<u32, Vec<u32>>, p_count: &mut usize, reg_a:u32, reg_b:u32, reg_c:u32){
  let segment = seg.get(&regs[reg_b as usize]).unwrap();
  let val = segment[regs[reg_c as usize] as usize];
  regs[reg_a as usize] = val;
  *p_count += 1;
}

///Private function to store values into segemented memory, in format m[r{}][r{}] := r{};", get(&RA, inst), get(&RB, inst), get(&RC, inst)
/// 
/// # Arguments
/// * `regs`: the registers in the UM
/// * `seg`: the segmented memory in the UM
/// * `p_count`: the program counter
/// * `reg_a`: a register number to point to in the UM
/// * `reg_b`: a register number to point to in the UM
/// * `reg_c`: a register number to point to in the UM
fn store(regs:&mut Vec<u32>, seg: &mut HashMap<u32, Vec<u32>>, p_count: &mut usize, reg_a:u32, reg_b:u32, reg_c:u32){
  let val = regs[reg_c as usize];
  let c = seg.get_mut(&regs[reg_a as usize]).unwrap();
  c[regs[reg_b as usize] as usize] = val;
  *p_count += 1;
}

///Private function to map segmented memory in format r{} := map segment (r{} words);", get(&RB, inst), get(&RC, inst)
/// 
/// # Arguments
/// * `regs`: the registers in the UM
/// * `seg`: the segmented memory in the UM
/// * `id_pool`: a register number to point to in the UM
/// * `s_ids`: the hightest ID segment to yet be mapped
/// * `p_count`: the program counter in the UM
/// * `reg_b`: a register number to point to in the UM
/// * `reg_c`: a register number to point to in the UM
fn map_segment(regs:&mut Vec<u32>, seg: &mut HashMap<u32, Vec<u32>>, id_pool: &mut Vec<u32>, s_ids:&mut u32, p_count: &mut usize,  reg_b:u32, reg_c:u32){
  if id_pool.len() > 0 {
    let id = id_pool.pop().unwrap();
    seg.insert(id, vec![0; regs[reg_c as usize] as usize]);
    regs[reg_b as usize] = id;
    *p_count += 1;
  }
  else{
    seg.insert(*s_ids, vec![0; regs[reg_c as usize] as usize]);
    regs[reg_b as usize] = *s_ids;
    *s_ids += 1;
    *p_count += 1;
  }
}

///Private function to unmap segmented memory in format unmap r{};", get(&RC, inst)
/// 
/// # Arguments
/// * `regs`: the registers in the UM
/// * `seg`: the segmented memory in the UM
/// * `p_count`: the program counter in the UM
/// * `id_pool`: the pool of used and unmapped segment ID's
/// * `reg_c`: a register number to point to in the UM
fn unmap_segment(regs: &mut Vec<u32>, p_count: &mut usize, id_pool: &mut Vec<u32>, reg_c:u32){
  id_pool.push(regs[reg_c as usize]);
  *p_count += 1;
}

/// Private function to get input  in format r{} := input();", get(&RC, inst)
/// 
/// # Arguments
/// * `regs`: the registers in the UM
/// * `p_count`: the program counter in the UM
/// * `reg_c`: a register number to point to in the UM
fn input(regs:&mut Vec<u32>, p_count: &mut usize, reg_c:u32){
  match io::stdin().bytes().next() {
    Some(value) => {
        regs[reg_c as usize] = value.unwrap() as u32;
        *p_count += 1;
    }
    None => {
      regs[reg_c as usize] = !0 as u32;
      *p_count += 1;
    }
  }
}
 
///Private function to load program in format goto r{} in program m[r{}];", get(&RC, inst), get(&RB, inst)
/// 
/// # Arguments
/// * `regs`: the registers in the UM
/// * `seg`: the segmented memory in the UM
/// * `p_count`: the program counter in the UM
/// * `reg_b`: a register number to point to in the UM
/// * `reg_c`: a register number to point to in the UM
fn load_program(regs: &mut Vec<u32>, seg: &mut HashMap<u32, Vec<u32>>, p_count:&mut usize, reg_b:u32, reg_c:u32){
  if regs[reg_b as usize] == 0 {
    *p_count = regs[reg_c as usize] as usize;
  }
  else{
    *seg.get_mut(&0).unwrap() = seg[&regs[reg_b as usize]].clone();
    *p_count = regs[reg_c as usize] as usize;
  }
}

///Public function to carry out instructions dervied from the UM words passed to it
/// 
/// # Arguments
/// * `inst`: the full 32 bit word instruction
/// * `um`: the universal machine, a struct that stores the registers, memory segments, program counter, segment ID and ID pool, and the program counter
pub fn do_instruction(inst: Umi, um:&mut UniversalMachine) {
    match get(&OPCODE, inst) {
        o if o == Opcode::CMov as u32 => {
          cmov(&mut um.registers, &mut um.program_counter, get(&RA, inst), get(&RB, inst), get(&RC, inst));
        },
        o if o == Opcode::Load as u32 => {
          load(&mut um.registers, &mut um.segments, &mut um.program_counter, get(&RA, inst), get(&RB, inst), get(&RC, inst));
        },
        o if o == Opcode::Store as u32 => {
          store(&mut um.registers, &mut um.segments, &mut um.program_counter, get(&RA, inst), get(&RB, inst), get(&RC, inst));
        },
        o if o == Opcode::Add as u32 => {
          add(&mut um.registers, &mut um.program_counter, get(&RA, inst), get(&RB, inst), get(&RC, inst));
        },
        o if o == Opcode::Mul as u32 => {
          multiply(&mut um.registers, &mut um.program_counter, get(&RA, inst), get(&RB, inst), get(&RC, inst));
        },
        o if o == Opcode::Div as u32 => {
          divide(&mut um.registers, &mut um.program_counter, get(&RA, inst), get(&RB, inst), get(&RC, inst));
        },
        // possible enhancement: if RB == RC, complement RC
        o if o == Opcode::Nand as u32 => {
          nand(&mut um.registers, &mut um.program_counter, get(&RA, inst), get(&RB, inst), get(&RC, inst));
        },
        o if o == Opcode::Halt as u32 => {
          exit(1);
        },
        o if o == Opcode::MapSegment as u32 => {
          map_segment(&mut um.registers, &mut um.segments, &mut um.id_pool, &mut um.segment_ids, &mut um.program_counter, get(&RB, inst), get(&RC, inst));
        },
        o if o == Opcode::UnmapSegment as u32 => {
          unmap_segment(&mut um.registers, &mut um.program_counter, &mut um.id_pool, get(&RC, inst));
        },
        o if o == Opcode::Output as u32 => {
          output(&mut um.registers, &mut um.program_counter, get(&RC, inst));
        },
        o if o == Opcode::Input as u32 => {
          input(&mut um.registers, &mut um.program_counter, get(&RC, inst));
        },
        o if o == Opcode::LoadProgram as u32 => {
          //format!("goto r{} in program m[r{}];", get(&RC, inst), get(&RB, inst))
          load_program(&mut um.registers, &mut um.segments, &mut um.program_counter, get(&RB, inst), get(&RC, inst));
        },
        o if o == Opcode::LoadValue as u32 => {
          load_value(&mut um.registers, &mut um.program_counter, get(&RL, inst), get(&VL, inst));
        },
        _ => {
          exit(2);
        }
    }
}