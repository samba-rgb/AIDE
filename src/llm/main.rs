mod phi_model;
mod command_processor;

use anyhow::Result;
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    // Read model config from environment or use defaults
    let base_url = std::env::var("OLLAMA_BASE_URL").unwrap_or_else(|_| "http://localhost:11434".to_string());
    let model_name = std::env::var("OLLAMA_MODEL_NAME").unwrap_or_else(|_| "qwen2.5-coder:0.5b".to_string());
    // Initialize the command processor (connects to Ollama)
    let mut processor = command_processor::CommandProcessor::new(base_url, model_name).await?;

    if args.len() > 1 {
        // Direct query mode - takes input and returns command
        let query = args[1..].join(" ");
        let command = processor.process_query(&query).await?;
        println!("{}", command);
    } else {
        // Interactive mode
        run_interactive_mode(&mut processor).await?;
    }

    Ok(())
}

async fn run_interactive_mode(processor: &mut command_processor::CommandProcessor) -> Result<()> {
    println!("🤖 Command Helper (powered by qwen2.5-coder:0.5b)");
    println!("Enter what you want to do and get the exact command!");
    println!("Type 'exit' to quit.\n");

    loop {
        print!("📝 What do you want to do? ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let query = input.trim();

        match query {
            "exit" => {
                println!("👋 Goodbye!");
                break;
            }
            "" => continue,
            _ => {
                print!("🧠 Generating command...");
                io::stdout().flush()?;
                
                match processor.process_query(query).await {
                    Ok(command) => {
                        println!("\r✅ Command: {}", command);
                    }
                    Err(e) => {
                        println!("\r❌ Error: {}", e);
                    }
                }
            }
        }
        println!();
    }

    Ok(())
}
