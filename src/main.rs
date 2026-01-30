mod crypto;
mod stego;
mod vault;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::*;
use image::GenericImageView;
use std::fs;
use std::path::{Path, PathBuf};
use vault::PasswordEntry;

#[derive(Parser)]
#[command(name = "pixelvault")]
#[command(author = "Étudiant en Droit")]
#[command(version = "0.2.0")]
#[command(
    about = "Gestionnaire de mots de passe par stéganographie",
    long_about = "Chaque mot de passe est dissimulé dans une image séparée, stockée dans le dossier ./vault"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Crée le dossier ./vault
    Init,

    /// Ajoute un mot de passe dans une nouvelle image
    Add {
        /// Image source à utiliser
        image: PathBuf,
        /// Nom du service (ex: facebook) - l'image sera sauvegardée sous vault/facebook.png
        name: String,
    },

    /// Récupère un mot de passe
    Get {
        /// Nom du service à récupérer (ex: facebook)
        name: String,
    },

    /// Liste tous les mots de passe stockés
    List,

    /// Supprime un mot de passe (supprime l'image)
    Remove {
        /// Nom du service à supprimer
        name: String,
    },
}

const VAULT_DIR: &str = "./vault";

fn main() {
    if let Err(e) = run() {
        eprintln!("{} {}", "Erreur:".red().bold(), e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => init_vault(),
        Commands::Add { image, name } => add_entry(&image, &name),
        Commands::Get { name } => get_entry(&name),
        Commands::List => list_entries(),
        Commands::Remove { name } => remove_entry(&name),
    }
}

/// Initialise le dossier du coffre-fort
fn init_vault() -> Result<()> {
    println!(
        "{}",
        "🔐 Initialisation du coffre-fort...".cyan().bold()
    );

    let path = Path::new(VAULT_DIR);
    if path.exists() {
        println!("{}", "⚠️  Le dossier ./vault existe déjà.".yellow());
    } else {
        fs::create_dir(path).context("Impossible de créer le dossier vault")?;
        println!("{}", "✅ Dossier ./vault créé avec succès !".green().bold());
    }

    Ok(())
}

/// Ajoute une entrée (crée une nouvelle image dans le vault)
fn add_entry(source_image: &PathBuf, name: &str) -> Result<()> {
    // Vérification du coffre
    let vault_path = Path::new(VAULT_DIR);
    if !vault_path.exists() {
        anyhow::bail!("Dossier vault introuvable. Lancez 'pixelvault init' d'abord.");
    }

    // Vérification de l'image source
    if !source_image.exists() {
        anyhow::bail!("Image source introuvable: {:?}", source_image);
    }

    // Définition du chemin de sortie
    let output_path = vault_path.join(format!("{}.png", name));
    if output_path.exists() {
        anyhow::bail!("L'entrée '{}' existe déjà !", name);
    }

    println!(
        "{}",
        format!("➕ Ajout de l'entrée : {}", name).cyan().bold()
    );

    // Collecte des infos
    println!("Nom d'utilisateur:");
    let mut username = String::new();
    std::io::stdin().read_line(&mut username)?;

    println!("Mot de passe (laissez vide pour générer):");
    let entry_password = rpassword::prompt_password("Mot de passe: ")?;
    let entry_password = if entry_password.is_empty() {
        generate_password()
    } else {
        entry_password
    };

    println!("URL (optionnel):");
    let mut url = String::new();
    std::io::stdin().read_line(&mut url)?;

    println!("Notes (optionnel):");
    let mut notes = String::new();
    std::io::stdin().read_line(&mut notes)?;

    // Création de l'objet
    let entry = PasswordEntry {
        name: name.to_string(),
        username: username.trim().to_string(),
        password: entry_password,
        url: if url.trim().is_empty() { None } else { Some(url.trim().to_string()) },
        notes: if notes.trim().is_empty() { None } else { Some(notes.trim().to_string()) },
    };

    // Sérialisation
    let entry_json = entry.to_json()?;

    // Demande du mot de passe maître pour chiffrer cette entrée spécifique
    println!(
        "\n{}",
        "Choisissez un mot de passe maître pour sécuriser cette image :".yellow()
    );
    let master_password = rpassword::prompt_password("Mot de passe maître: ")?;
    let confirm = rpassword::prompt_password("Confirmez: ")?;

    if master_password != confirm {
        anyhow::bail!("Les mots de passe ne correspondent pas");
    }

    let img = image::open(source_image).context("Erreur lecture image source")?;
    let (_width, _height) = img.dimensions();

    let salt = crypto::generate_salt();
    let key = crypto::derive_key(&master_password, &salt)?;
    let encrypted = crypto::encrypt(entry_json.as_bytes(), &key)?;

    let mut payload = Vec::new();
    payload.extend_from_slice(&salt);
    payload.extend_from_slice(&encrypted);

    let stego_img = stego::encode(&img, &payload)?;
    stego_img.save(&output_path)?;

    println!("\n{}", "✅ Entrée sauvegardée avec succès!".green().bold());
    println!("🖼️  Image créée : {:?}", output_path);

    Ok(())
}

/// Récupère une entrée
fn get_entry(name: &str) -> Result<()> {
    let file_path = Path::new(VAULT_DIR).join(format!("{}.png", name));

    if !file_path.exists() {
        anyhow::bail!("Aucune entrée trouvée avec le nom '{}'", name);
    }

    let password = rpassword::prompt_password("Mot de passe maître: ")?;

    let img = image::open(&file_path).context("Erreur lecture image")?;
    let payload = stego::decode(&img)?;

    if payload.len() < 16 {
        anyhow::bail!("Données corrompues ou image non valide");
    }

    let (salt, encrypted) = payload.split_at(16);
    let key = crypto::derive_key(&password, salt)?;

    let decrypted = crypto::decrypt(encrypted, &key)
        .context("Mot de passe incorrect ou données corrompues")?;

    let json = String::from_utf8(decrypted)?;
    let entry = PasswordEntry::from_json(&json)?;

    println!("\n{}", format!("🔑 Entrée: {}", entry.name).cyan().bold());
    println!("👤 Utilisateur: {}", entry.username);
    println!("🔒 Mot de passe: {}", entry.password.green());
    if let Some(url) = &entry.url {
        println!("🌐 URL: {}", url);
    }
    if let Some(notes) = &entry.notes {
        println!("📝 Notes: {}", notes);
    }

    Ok(())
}

/// Liste les fichiers dans ./vault
fn list_entries() -> Result<()> {
    let path = Path::new(VAULT_DIR);
    if !path.exists() {
        println!("{}", "Le coffre n'est pas initialisé (dossier vault manquant)".yellow());
        return Ok(());
    }

    println!("{}", "📚 Contenu du coffre :".cyan().bold());
    let mut count = 0;

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("png") {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                println!("  • {}", stem);
                count += 1;
            }
        }
    }

    if count == 0 {
        println!("  (Vide)");
    } else {
        println!("\nTotal : {} entrée(s)", count);
    }

    Ok(())
}

/// Supprime une entrée
fn remove_entry(name: &str) -> Result<()> {
    let file_path = Path::new(VAULT_DIR).join(format!("{}.png", name));

    if !file_path.exists() {
        anyhow::bail!("Entrée '{}' introuvable", name);
    }

    fs::remove_file(&file_path)?;
    println!("{}", format!("🗑️  Entrée '{}' supprimée.", name).yellow());

    Ok(())
}

fn generate_password() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*";
    let mut rng = rand::thread_rng();
    (0..20)
        .map(|_| CHARSET[rng.gen_range(0..CHARSET.len())] as char)
        .collect()
}
