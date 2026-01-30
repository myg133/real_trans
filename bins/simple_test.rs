//! 简单测试程序，验证双向翻译器的基本功能

use real_trans::bidirectional_translator::BidirectionalTranslator;

#[tokio::main]
async fn main() {
    println!("=== 双向翻译器基本功能测试 ===\n");

    // 1. 创建双向翻译器 (中文 -> 英文)
    println!("1. 创建双向翻译器 (zh -> en)...");
    let mut translator = BidirectionalTranslator::new("zh", "en")
        .expect("Failed to create bidirectional translator");

    // 2. 显示当前语言对
    let lang_pair = translator.get_current_language_pair();
    println!("   当前语言对: {} -> {}", lang_pair.source, lang_pair.target);

    // 3. 更新语言对 (英文 -> 法文)
    println!("\n2. 更新语言对为 (en -> fr)...");
    translator.update_language_pair("en", "fr")
        .expect("Failed to update language pair");

    let new_lang_pair = translator.get_current_language_pair();
    println!("   新语言对: {} -> {}", new_lang_pair.source, new_lang_pair.target);

    // 4. 检查运行状态
    println!("\n3. 检查运行状态...");
    println!("   是否运行: {}", translator.is_running());

    // 5. 设置结果回调
    println!("\n4. 设置翻译结果回调...");
    translator.set_result_callback(|result| {
        match result.direction {
            real_trans::bidirectional_translator::TranslationDirection::UserToOther => {
                println!("   用户->对方: {} -> {}", result.original_text, result.translated_text);
            }
            real_trans::bidirectional_translator::TranslationDirection::OtherToUser => {
                println!("   对方->用户: {} -> {}", result.original_text, result.translated_text);
            }
        }
    });

    println!("\n=== 测试完成 ===");
    println!("\n双向翻译器功能:");
    println!("- 支持语言对设置");
    println!("- 支持动态语言切换");
    println!("- 支持双向翻译方向");
    println!("- 支持结果回调处理");
    println!("- 支持模拟音频输入");
}