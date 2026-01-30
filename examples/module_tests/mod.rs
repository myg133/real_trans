//! 模块化测试套件
//! 提供逐步测试全双工实时双向语音同传系统各组件的功能

pub mod input_test;
pub mod input_with_translation_test;
pub mod output_test;
pub mod full_integration_test;

/// 测试套件枚举
#[derive(Debug, Clone)]
pub enum ModuleTest {
    InputTest,
    InputWithTranslationTest,
    OutputTest,
    FullIntegrationTest,
}

impl ModuleTest {
    /// 获取测试名称
    pub fn name(&self) -> &'static str {
        match self {
            ModuleTest::InputTest => "音频输入模块测试",
            ModuleTest::InputWithTranslationTest => "音频输入+翻译模块测试",
            ModuleTest::OutputTest => "音频输出模块测试",
            ModuleTest::FullIntegrationTest => "完整集成测试",
        }
    }

    /// 获取测试描述
    pub fn description(&self) -> &'static str {
        match self {
            ModuleTest::InputTest => "测试物理麦克风输入并将音频保存到文件",
            ModuleTest::InputWithTranslationTest => "测试物理麦克风输入，经过翻译后保存到文件",
            ModuleTest::OutputTest => "测试从文件读取音频并通过耳机播放",
            ModuleTest::FullIntegrationTest => "结合输入、翻译和输出模块的端到端测试",
        }
    }
}

/// 运行指定的测试
pub async fn run_test(test: ModuleTest) -> Result<(), Box<dyn std::error::Error>> {
    println!("=================================");
    println!("开始运行: {}", test.name());
    println!("描述: {}", test.description());
    println!("=================================\n");

    match test {
        ModuleTest::InputTest => run_input_test().await?,
        ModuleTest::InputWithTranslationTest => run_input_with_translation_test().await?,
        ModuleTest::OutputTest => run_output_test()?,
        ModuleTest::FullIntegrationTest => run_full_integration_test().await?,
    }

    println!("\n=================================");
    println!("测试完成: {}", test.name());
    println!("=================================\n");

    Ok(())
}

/// 运行音频输入测试
async fn run_input_test() -> Result<(), Box<dyn std::error::Error>> {
    // 这里我们会调用具体的测试逻辑
    // 由于每个测试都在单独的文件中，我们只做占位符
    println!("音频输入测试 - 请运行: cargo run --example input_test");
    Ok(())
}

/// 运行音频输入+翻译测试
async fn run_input_with_translation_test() -> Result<(), Box<dyn std::error::Error>> {
    println!("音频输入+翻译测试 - 请运行: cargo run --example input_with_translation_test");
    Ok(())
}

/// 运行音频输出测试
fn run_output_test() -> Result<(), Box<dyn std::error::Error>> {
    println!("音频输出测试 - 请运行: cargo run --example output_test");
    Ok(())
}

/// 运行完整集成测试
async fn run_full_integration_test() -> Result<(), Box<dyn std::error::Error>> {
    println!("完整集成测试 - 请运行: cargo run --example full_integration_test");
    Ok(())
}