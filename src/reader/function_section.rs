use std::io;

use crate::{reader::Section, utils, Error};

pub struct FunctionSection {
    pub funcs: Vec<usize>,
}

impl Section for FunctionSection {
    fn read<R: io::Read>(reader: &mut R) -> Result<FunctionSection, Error> {
        let funcs = utils::read_vec(reader, |r| Ok(utils::read_leb128_u32(r)? as usize))?;

        Ok(FunctionSection { funcs })
    }
}
