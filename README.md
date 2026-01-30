# 🔐 PixelVault - Proof of concept

**Votre coffre-fort numérique, caché dans des pixels.**

PixelVault est un gestionnaire de mots de passe unique qui utilise la **stéganographie**. Au lieu de stocker vos mots de passe dans une base de données évidente, il les dissimule à l'intérieur d'images PNG ordinaires.

---

## 🚀 Installation & Lancement

Ce projet est conçu pour fonctionner sur **Linux**, **Windows** et **macOS**.
Vous devez simplement avoir **[Rust](https://rustup.rs/)** installé sur votre machine.

### 1. Télécharger
Téléchargez ce dossier et désarchivez-le.

### 2. Lancer
Ne vous embêtez pas avec la ligne de commande. Double-cliquez simplement sur le lanceur correspondant à votre système :

*   🐧 **Linux** : Double-cliquez sur **`run_linux.sh`**.
*   🪟 **Windows** : Double-cliquez sur **`run_windows.bat`**.
*   🍎 **macOS** : Double-cliquez sur **`run_macos.command`**.

*Note : Au premier lancement, le programme va se compiler (se fabriquer) automatiquement. Cela peut prendre quelques minutes.*

---

## 🔑 Utilisation

Une fois lancé, vous arriverez sur un terminal interactif.

### 1. Créer le coffre
La première fois, tapez cette commande pour initialiser votre dossier sécurisé :
```bash
pixelvault init
```

### 2. Ranger un mot de passe
Pour cacher un mot de passe, il vous faut une "image source" (n'importe quelle image PNG sur votre ordinateur).
Supposons que vous ayez `photo.png` et que vous vouliez y cacher votre mot de passe Facebook :

```bash
pixelvault add photo.png facebook
```

Le programme va vous demander les infos (identifiant, mot de passe...) et créer une nouvelle image **`vault/facebook.png`**.
C'est cette image qui contient votre secret !

### 3. Récupérer un secret
Besoin de votre mot de passe Facebook ?

```bash
pixelvault get facebook
```
Le programme vous demandera le mot de passe maître, décodera l'image, et affichera vos infos.

### 4. Voir tout le contenu
```bash
pixelvault list
```

---

**PixelVault - POC**
*Projet éducatif et open-source.*
