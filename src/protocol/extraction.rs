use std::io::Read;
use protocol::primitive_parse::parse_u16l;
use std::io::Error as IOError;

const LURK_FIXED_STRING_SIZE: u16 = 32;

pub enum ExtractNode {
    ByteChunk(u16),
    FixedStringChunk,
    VarStringChunk,
    FracturedVarString(u16),
}

pub struct Extractor {
    pattern_nodes: Vec<ExtractNode>,
}

impl Extractor {
    pub fn create(nodes: Vec<ExtractNode>) -> Extractor {
        Extractor {
            pattern_nodes: nodes,
        }
    }

    pub fn message() -> Extractor {
        Extractor::create(vec![
            ExtractNode::FracturedVarString(2 * LURK_FIXED_STRING_SIZE),
        ])
    }

    pub fn change_room() -> Extractor {
        Extractor::create(vec![ExtractNode::ByteChunk(2)])
    }

    pub fn fight() -> Extractor {
        Extractor::create(vec![])
    }

    pub fn pvp_fight() -> Extractor {
        Extractor::create(vec![ExtractNode::FixedStringChunk])
    }

    pub fn loot() -> Extractor {
        Extractor::create(vec![ExtractNode::FixedStringChunk])
    }

    pub fn start() -> Extractor {
        Extractor::create(vec![])
    }

    pub fn error() -> Extractor {
        Extractor::create(vec![ExtractNode::ByteChunk(1), ExtractNode::VarStringChunk])
    }

    pub fn accept() -> Extractor {
        Extractor::create(vec![ExtractNode::ByteChunk(1)])
    }

    pub fn room() -> Extractor {
        Extractor::create(vec![
            ExtractNode::ByteChunk(2),
            ExtractNode::FixedStringChunk,
            ExtractNode::VarStringChunk,
        ])
    }

    pub fn character() -> Extractor {
        Extractor::create(vec![
            ExtractNode::FixedStringChunk,
            ExtractNode::ByteChunk(13),
            ExtractNode::VarStringChunk,
        ])
    }

    pub fn game() -> Extractor {
        Extractor::create(vec![ExtractNode::ByteChunk(4), ExtractNode::VarStringChunk])
    }

    pub fn leave() -> Extractor {
        Extractor::create(vec![])
    }

    pub fn connection() -> Extractor {
        Extractor::create(vec![
            ExtractNode::ByteChunk(2),
            ExtractNode::FixedStringChunk,
            ExtractNode::VarStringChunk,
        ])
    }

    pub fn extract<F>(&self, stream: &mut F) -> Result<Vec<u8>, IOError>
    where
        F: Read,
    {
        let mut result: Vec<u8> = vec![];

        for node in self.pattern_nodes.iter() {
            let data: Vec<u8> = match node {
                &ExtractNode::ByteChunk(size) => self.extract_byte_chunk(stream, size)?,
                &ExtractNode::FixedStringChunk => self.extract_fixed_string_chunk(stream)?,
                &ExtractNode::VarStringChunk => self.extract_var_string_chunk(stream)?,
                &ExtractNode::FracturedVarString(gap_size) => {
                    self.extract_fractured_var_string_chunk(stream, gap_size)?
                }
            };

            result.extend(&data);
        }

        Ok(result)
    }

    fn extract_byte_chunk<F>(&self, stream: &mut F, n: u16) -> Result<Vec<u8>, IOError>
    where
        F: Read,
    {
        let mut buffer: Vec<u8> = vec![0u8; n as usize];
        stream.read_exact(&mut buffer)?;
        Ok(buffer)
    }

    fn extract_fixed_string_chunk<F>(&self, stream: &mut F) -> Result<Vec<u8>, IOError>
    where
        F: Read,
    {
        let mut buffer: Vec<u8> = vec![0u8; LURK_FIXED_STRING_SIZE as usize];
        stream.read_exact(&mut buffer)?;
        Ok(buffer)
    }

    fn extract_var_string_chunk<F>(&self, stream: &mut F) -> Result<Vec<u8>, IOError>
    where
        F: Read,
    {
        let mut buffer: Vec<u8> = vec![0u8; 2];
        stream.read_exact(&mut buffer)?;
        let len = parse_u16l(buffer.as_slice());
        let mut data_buffer: Vec<u8> = vec![0u8; len as usize];
        stream.read_exact(&mut data_buffer)?;
        buffer.extend(&data_buffer);
        Ok(buffer)
    }

    fn extract_fractured_var_string_chunk<F>(
        &self,
        stream: &mut F,
        n: u16,
    ) -> Result<Vec<u8>, IOError>
    where
        F: Read,
    {
        let mut buffer: Vec<u8> = vec![0u8; n as usize + 2];
        stream.read_exact(&mut buffer)?;
        let len = parse_u16l(buffer.as_slice());
        let mut data_buffer: Vec<u8> = vec![0u8; len as usize];
        stream.read_exact(&mut data_buffer)?;
        buffer.extend(&data_buffer);
        Ok(buffer)
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use std::io::BufReader;

    #[test]
    fn test_byte_chunk_extraction() {
        let data = vec![0u8, 0u8, 0u8, 0u8];
        let mut readable = BufReader::new(data.as_slice());
        let extractor = Extractor::create(vec![ExtractNode::ByteChunk(4)]);
        let data_result = extractor.extract_byte_chunk(&mut readable, 4);
        assert!(data_result.is_ok());
        assert_eq!(data_result.unwrap().len(), data.len());
    }

    #[test]
    fn test_fixed_string_chunk_extraction() {
        let data = vec![0u8; LURK_FIXED_STRING_SIZE as usize];
        let mut readable = BufReader::new(data.as_slice());
        let extractor = Extractor::create(vec![ExtractNode::FixedStringChunk]);
        let data_result = extractor.extract_fixed_string_chunk(&mut readable);
        assert!(data_result.is_ok());
        assert_eq!(data_result.unwrap().len(), data.len());
    }

    #[test]
    fn test_var_string_chunk_extraction() {
        let data = vec![4u8, 0u8, 0u8, 0u8, 0u8, 0u8];
        let mut readable = BufReader::new(data.as_slice());
        let extractor = Extractor::create(vec![ExtractNode::VarStringChunk]);
        let data_result = extractor.extract_var_string_chunk(&mut readable);
        assert!(data_result.is_ok());
        assert_eq!(data_result.unwrap().len(), data.len());
    }

    #[test]
    fn test_fractured_var_string_chunk_extraction() {
        let data = vec![4u8, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut readable = BufReader::new(data.as_slice());
        let extractor = Extractor::create(vec![ExtractNode::FracturedVarString(4)]);
        let data_result = extractor.extract_fractured_var_string_chunk(&mut readable, 4);
        assert!(data_result.is_ok());
        assert_eq!(data_result.unwrap().len(), data.len());
    }

    #[test]
    fn test_message_extraction() {
        let mut data = vec![4u8, 0u8];
        data.extend(vec![0u8; 68]);

        let mut readable = BufReader::new(data.as_slice());
        let extractor = Extractor::message();
        let data_result = extractor.extract(&mut readable);
        assert!(data_result.is_ok());
        assert_eq!(data_result.unwrap().len(), data.len());
    }

    #[test]
    fn test_change_room_extraction() {
        let data = vec![0u8; 2];

        let mut readable = BufReader::new(data.as_slice());
        let extractor = Extractor::change_room();
        let data_result = extractor.extract(&mut readable);
        assert!(data_result.is_ok());
        assert_eq!(data_result.unwrap().len(), data.len());
    }

    #[test]
    fn test_fight_extraction() {
        let data = vec![];

        let mut readable = BufReader::new(data.as_slice());
        let extractor = Extractor::fight();
        let data_result = extractor.extract(&mut readable);
        assert!(data_result.is_ok());
        assert_eq!(data_result.unwrap().len(), data.len());
    }

    #[test]
    fn test_pvp_fight_extraction() {
        let data = vec![0u8; LURK_FIXED_STRING_SIZE as usize];

        let mut readable = BufReader::new(data.as_slice());
        let extractor = Extractor::pvp_fight();
        let data_result = extractor.extract(&mut readable);
        assert!(data_result.is_ok());
        assert_eq!(data_result.unwrap().len(), data.len());
    }

    #[test]
    fn test_loot_extraction() {
        let data = vec![0u8; LURK_FIXED_STRING_SIZE as usize];

        let mut readable = BufReader::new(data.as_slice());
        let extractor = Extractor::loot();
        let data_result = extractor.extract(&mut readable);
        assert!(data_result.is_ok());
        assert_eq!(data_result.unwrap().len(), data.len());
    }

    #[test]
    fn test_start_extraction() {
        let data = vec![];

        let mut readable = BufReader::new(data.as_slice());
        let extractor = Extractor::start();
        let data_result = extractor.extract(&mut readable);
        assert!(data_result.is_ok());
        assert_eq!(data_result.unwrap().len(), data.len());
    }

    #[test]
    fn test_error_extraction() {
        let data = vec![0u8, 4u8, 0u8, 0u8, 0u8, 0u8, 0u8];

        let mut readable = BufReader::new(data.as_slice());
        let extractor = Extractor::error();
        let data_result = extractor.extract(&mut readable);
        assert!(data_result.is_ok());
        assert_eq!(data_result.unwrap().len(), data.len());
    }

    #[test]
    fn test_accept_extraction() {
        let data = vec![0u8];

        let mut readable = BufReader::new(data.as_slice());
        let extractor = Extractor::accept();
        let data_result = extractor.extract(&mut readable);
        assert!(data_result.is_ok());
        assert_eq!(data_result.unwrap().len(), data.len());
    }

    #[test]
    fn test_room_extraction() {
        let mut data = vec![0u8; 34];
        data.extend(vec![4u8, 0u8, 0u8, 0u8, 0u8, 0u8]);

        let mut readable = BufReader::new(data.as_slice());
        let extractor = Extractor::room();
        let data_result = extractor.extract(&mut readable);
        assert!(data_result.is_ok());
        assert_eq!(data_result.unwrap().len(), data.len());
    }

    #[test]
    fn test_character_extraction() {
        let mut data = vec![0u8; 45];
        data.extend(vec![4u8, 0u8, 0u8, 0u8, 0u8, 0u8]);

        let mut readable = BufReader::new(data.as_slice());
        let extractor = Extractor::character();
        let data_result = extractor.extract(&mut readable);
        assert!(data_result.is_ok());
        assert_eq!(data_result.unwrap().len(), data.len());
    }

    #[test]
    fn test_game_extraction() {
        let mut data = vec![0u8; 4];
        data.extend(vec![4u8, 0u8, 0u8, 0u8, 0u8, 0u8]);

        let mut readable = BufReader::new(data.as_slice());
        let extractor = Extractor::game();
        let data_result = extractor.extract(&mut readable);
        assert!(data_result.is_ok());
        assert_eq!(data_result.unwrap().len(), data.len());
    }

    #[test]
    fn test_leave_extraction() {
        let data = vec![];

        let mut readable = BufReader::new(data.as_slice());
        let extractor = Extractor::leave();
        let data_result = extractor.extract(&mut readable);
        assert!(data_result.is_ok());
        assert_eq!(data_result.unwrap().len(), data.len());
    }

    #[test]
    fn test_connection_extraction() {
        let mut data = vec![0u8; 34];
        data.extend(vec![4u8, 0u8, 0u8, 0u8, 0u8, 0u8]);

        let mut readable = BufReader::new(data.as_slice());
        let extractor = Extractor::connection();
        let data_result = extractor.extract(&mut readable);
        assert!(data_result.is_ok());
        assert_eq!(data_result.unwrap().len(), data.len());
    }
}
