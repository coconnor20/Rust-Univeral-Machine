use std::collections::HashMap;

use crate::rum_instruction;

type SegmentID = u32;

#[derive(Clone, Debug)]
pub struct UniversalMachine {
    pub registers:Vec<u32>,
    pub segments:HashMap<SegmentID, Vec<u32>>,
    pub id_pool:Vec<u32>,
    pub segment_ids:u32,
    pub program_counter:usize
}

impl UniversalMachine {
    pub fn new(instructions: Vec<u32>) -> Self{
        UniversalMachine {
            registers:vec![0;8],
            segments: HashMap::from([(0,instructions)]),
            id_pool: Vec::new(),
            segment_ids: 1,
            program_counter:0
        }
    }
}

/// Public function to take in program, intialize the um, and run the emulation cycle
/// 
/// # Arguments
/// * `instructions`: A vector of 32 bit codewords that comprise the instrunctions of the program
#[inline(always)]
pub fn run_um(instructions: Vec<u32>){
    let mut um = UniversalMachine::new(instructions);
    loop {
        rum_instruction::do_instruction(um.segments[&0][um.program_counter], &mut um);
    }
}


#[cfg(test)]
mod tests{
    use crate::rumload;
    use crate::emulation_unit;

    #[test]
    fn test_cat(){
        let instructions = rumload::load(Some("tests/sandmark.umz"));
        emulation_unit::run_um(instructions);
    }
}