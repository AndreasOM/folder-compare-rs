#[derive(Debug)]
pub enum Message {
	Started(u64, u64),
	Progress(usize),
	FileDone,
    Success,
    Failure,
    Done,
}
