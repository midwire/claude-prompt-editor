use crate::parser;
use crate::parser::ast::PromptAst;
use crate::parser::serializer;

#[tauri::command]
pub fn parse_prompt(path: String) -> Result<PromptAst, String> {
    let file = crate::commands::file::read_prompt_file(&std::path::PathBuf::from(&path))?;
    parser::parse(&file.content)
}

#[tauri::command]
pub fn parse_content(content: String) -> Result<PromptAst, String> {
    parser::parse(&content)
}

#[tauri::command]
pub fn serialize_ast(ast: PromptAst) -> String {
    serializer::serialize(&ast)
}
