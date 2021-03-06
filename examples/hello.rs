mod flags {
    xflags::xflags! {
        cmd hello
            required name: String
        {
            optional -e, --emoji
        }
    }
}

fn main() {
    match flags::Hello::from_env() {
        Ok(flags) => {
            let bang = if flags.emoji { "❣️" } else { "!" };
            println!("Hello {}{}", flags.name, bang);
        }
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(1)
        }
    }
}
