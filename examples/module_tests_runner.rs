//! 模块化测试运行器
//! 用于运行全双工实时双向语音同传系统的模块化测试

use std::io::{self, Write};
use real_trans::examples::module_tests::{ModuleTest, run_test};

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

    print!("请选择要运行的测试 (1-5): ");
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let choice = input.trim().parse::<u32>().unwrap_or(0);

    match choice {
        1 => {
            run_test(ModuleTest::InputTest).await?;
        },
        2 => {
            run_test(ModuleTest::InputWithTranslationTest).await?;
        },
        3 => {
            run_test(ModuleTest::OutputTest)?;
        },
        4 => {
            run_test(ModuleTest::FullIntegrationTest).await?;
        },
        5 => {
            println!("运行所有测试...\n");
            
            run_test(ModuleTest::InputTest).await?;
            run_test(ModuleTest::InputWithTranslationTest).await?;
            run_test(ModuleTest::OutputTest)?;
            run_test(ModuleTest::FullIntegrationTest).await?;
            
            println!("所有测试完成！");
        },
        _ => {
            println!("无效选择，运行单个测试示例...");
            println!("您也可以直接运行特定测试：");
            println!("cargo run --example input_test");
            println!("cargo run --example input_with_translation_test");
            println!("cargo run --example output_test");
            println!("cargo run --example full_integration_test");
        }
    }

    Ok(())
}