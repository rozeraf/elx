use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "elx")]
#[command(about = "A personal ls-clone in Rust", long_about = None)]
pub struct Cli {
    /// Путь к директории или файлу (по умолчанию текущая директория)
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Использовать подробный формат вывода (права, владелец, размер, дата)
    #[arg(short = 'l', long)]
    pub long: bool,

    /// Показывать скрытые файлы (начинающиеся с точки)
    #[arg(short = 'a', long)]
    pub all: bool,

    /// Отображать содержимое в виде дерева
    #[arg(short = 'T', long)]
    pub tree: bool,

    /// Максимальная глубина рекурсии для древовидного отображения
    #[arg(long)]
    pub depth: Option<usize>,

    /// Не отображать иконки
    #[arg(long)]
    pub no_icons: bool,

    /// Не использовать цвета в выводе
    #[arg(long)]
    pub no_color: bool,
}
