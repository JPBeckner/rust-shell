use std::io::{Write, stdin, stdout};
use std::process::{Command, Stdio, Child};
use std::path::{Path};
use std::env;


fn main() {

    'shell_loop: loop {
        // use the `>` character as the prompt
        // need to explicitly flush this to ensure it prints before read_line
        print!("> ");
        stdout().flush().expect("Invalid input.");

        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();
                
        // must be peekable so we know when we are on the last command
        let mut commands = input.trim().split(" | ").peekable();
        let mut previous_command = None;

        while let Some(command) = commands.next() {

            // everything after the first whitespace character 
            //     is interpreted as args to the command
            // read_line leaves a trailing newline, wich trim removes
            // unwrap_or handle it if there is no input
            let mut parts = command.trim().split_whitespace();
            let command = parts.next().unwrap_or("");
            let args = parts;

            match command {
                "exit" => {
                    println!("Rust Shell terminated.");
                    return;
                },
                "" => continue 'shell_loop,
                "cd" => {
                    // default to '/' as new directory if one was not provided
                    let new_dir = args
                    .peekable()
                    .peek()
                    .map_or("/", |x| *x);
                    
                    let root = Path::new(new_dir);
                    if let Err(e) = env::set_current_dir(&root){
                        eprintln!("{}", e)
                    }
                    previous_command = None;
                },
                command => {
                    let stdin = previous_command
                    .map_or(
                        Stdio::inherit(),
                        |output: Child| Stdio::from(output.stdout.unwrap())
                    );

                    let stdout = if commands.peek().is_some() {
                        // there is another command piped behind this one
                        // prepare to send output to the next command
                        Stdio::piped()
                    } else {
                        // there are no more commands piped behind this one
                        // send output to shell stdout
                        Stdio::inherit()
                    };

                    let output = Command::new(command)
                    .args(args)
                    .stdin(stdin)
                    .stdout(stdout)
                    .spawn();
                    match output {
                        Ok(output) => { previous_command = Some(output); },
                        Err(e) => {
                            previous_command = None;
                            eprintln!("{}", e);
                        },
                    };
    
                },
            }

        }

        if let Some(mut final_command) = previous_command {
            // block until the final command has finished
            final_command.wait().expect("Could not wait.");
        }
        
    }

}
