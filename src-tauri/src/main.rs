// Previne a abertura de uma janela de terminal extra no Windows em modo release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::Command;
use tauri::Manager;            
use tauri::path::BaseDirectory; 

#[tauri::command]
fn gravar_firmware_bancada(
    app_handle: tauri::AppHandle, 
    serial_number: String, 
    hardware_version: String, 
    mosfet_pin: String
) -> Result<String, String> {

    let resource_path = app_handle
        .path()
        .resolve("resources/upload", BaseDirectory::Resource)
        .map_err(|e| format!("Não foi possível encontrar o script 'upload' no diretório resources: {}", e))?;

    let canal = serial_number.chars().skip(2).take(3).collect::<String>();
    let firmware_id = serial_number.chars().skip(7).collect::<String>();

    let ch_arg = canal.trim_start_matches('0').to_string();
    let fi_arg = firmware_id.trim_start_matches('0').to_string();
    
    let hw_base = if hardware_version.contains("1_0") { "1_0" } else { "1_5" };

    let device_id = format!("CH{:0>3}FI{:0>6}", 
        ch_arg.parse::<u32>().unwrap_or(0), 
        fi_arg.parse::<u32>().unwrap_or(0)
    );

    let firmware_name = if !mosfet_pin.is_empty() {
        format!("FI_{}_400", hw_base)
    } else {
        format!("FI_{}", hw_base)
    };

    let mosfet_arg = mosfet_pin;

    // CORRIGIDO: Agora a string está declarada e finalizada perfeitamente
    let seed_secret_env = "CHAVI".to_string();

    let mut comando = Command::new("sh");
    comando.arg(&resource_path)
           .arg(device_id)      
           .arg(firmware_name)  
           .arg(mosfet_arg);    

    // INJEÇÃO DA VARIÁVEL DE AMBIENTE DENTRO DO SUBPROCESSO DO TAURI
    comando.env("SEED_SECRET", seed_secret_env);

    if let Some(pasta_resources) = resource_path.parent() {
        comando.current_dir(pasta_resources);
    }

    let output = comando.output().map_err(|e| format!("Falha crítica ao disparar o script upload: {}", e))?;

    if output.status.success() {
        let resultado = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(resultado)
    } else {
        let erro = String::from_utf8_lossy(&output.stderr).to_string();
        let saida_comum = String::from_utf8_lossy(&output.stdout).to_string();
        
        // Retorna tanto o erro quanto a saída normal para facilitar o diagnóstico na tela
        Err(format!("Erro: {}\nSaída: {}", erro, saida_comum))
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            gravar_firmware_bancada
        ])
        .run(tauri::generate_context!())
        .expect("erro ao rodar a aplicação Tauri");
}