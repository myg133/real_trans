//! 模块化测试运行器
//! 用于运行全双工实时双向语音同传系统的模块化测试

use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=================================");
    println!("全双工实时双向语音同传系统 - 模块化测试");
    println!("=================================\n");

    println!("可用测试:");
    println!("1. 音频输入模块测试 - 测试物理麦克风输入并将音频保存到文件");
    println!("2. 音频输入+翻译模块测试 - 测试物理麦克风输入，经过翻译后保存到文件");
    println!("3. 音频输出模块测试 - 测试从文件读取音频并通过耳机播放");
    println!("4. 完整集成测试 - 结合输入、翻译和输出模块的端到端测试");
    println!("5. 运行所有测试\n");

    print!("请选择要运行的测试 (1-5)，或直接运行对应示例:\n");
    println!("cargo run --example input_test");
    println!("cargo run --example input_with_translation_test");
    println!("cargo run --example output_test");
    println!("cargo run --example full_integration_test");
    print!("\n请输入选择 (1-5): ");
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let choice = input.trim().parse::<u32>().unwrap_or(0);

    match choice {
        1 => {
            println!("运行音频输入模块测试...");
            println!("执行: cargo run --example input_test");
            std::process::Command::new("cargo")
                .args(&["run", "--example", "input_test"])
                .spawn()?
                .wait()
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
        },
        2 => {
            println!("运行音频输入+翻译模块测试...");
            println!("执行: cargo run --example input_with_translation_test");
            std::process::Command::new("cargo")
                .args(&["run", "--example", "input_with_translation_test"])
                .spawn()?
                .wait()
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
        },
        3 => {
            println!("运行音频输出模块测试...");
            println!("执行: cargo run --example output_test");
            std::process::Command::new("cargo")
                .args(&["run", "--example", "output_test"])
                .spawn()?
                .wait()
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
        },
        4 => {
            println!("运行完整集成测试...");
            println!("执行: cargo run --example full_integration_test");
            std::process::Command::new("cargo")
                .args(&["run", "--example", "full_integration_test"])
                .spawn()?
                .wait()
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
        },
        5 => {
            println!("运行所有测试...\n");
            
            println!("1. 运行音频输入模块测试...");
            std::process::Command::new("cargo")
                .args(&["run", "--example", "input_test"])
                .spawn()?
                .wait()?;
                
            println!("\n2. 运行音频输入+翻译模块测试...");
            std::process::Command::new("cargo")
                .args(&["run", "--example", "input_with_translation_test"])
                .spawn()?
                .wait()?;
                
            println!("\n3. 运行音频输出模块测试...");
            std::process::Command::new("cargo")
                .args(&["run", "--example", "output_test"])
                .spawn()?
                .wait()?;
                
            println!("\n4. 运行完整集成测试...");
            std::process::Command::new("cargo")
                .args(&["run", "--example", "full_integration_test"])
                .spawn()?
                .wait()?;
                
            println!("\n所有测试完成！");
        },
        _ => {
            println!("无效选择，您可以直接运行特定测试：");
            println!("cargo run --example input_test");
            println!("cargo run --example input_with_translation_test");
            println!("cargo run --example output_test");
            println!("cargo run --example full_integration_test");
        }
    }

    Ok(())
}