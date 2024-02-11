mod cli;
mod engine;
mod parser;

fn main() {
	println!("Type an arithmetic expression and press Enter to evaluate. Press Ctrl+C to exit.\n");
	let mut buffer = String::new();
	loop {
		match cli::try_calculate(&mut buffer) {
			Ok(result) => println!("{result}\n"),
			Err(error) => println!("{error}\n"),
		}
		buffer.clear();
	}
}
