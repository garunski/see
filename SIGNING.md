# Code Signing Setup Guide

This guide explains how to set up code signing for macOS and Windows releases.

## macOS Code Signing & Notarization

### Prerequisites

1. **Apple Developer Program membership** ($99/year)
   - Enroll at https://developer.apple.com/programs/
   - Choose Individual or Organization account
   - Wait for approval (instant to 3 days)

### Step 1: Create Developer ID Certificate

> **CRITICAL:** You must create the certificate request from **Keychain Access on THIS Mac** - not from another computer or a downloaded file. This ensures the private key is generated and stored on your Mac.

1. **Request Certificate from Apple (on THIS Mac):**
   - Open **Keychain Access** on your Mac (Applications → Utilities → Keychain Access)
   - In the left sidebar under "Keychains", select **"login"**
   - In the left sidebar under "Category", select **"Certificates"** (the central area items don't matter)
   - From the menu bar at the top: `Keychain Access` → `Certificate Assistant` → `Request a Certificate from a Certificate Authority`
   - Fill in:
     - **User Email Address:** Your Apple ID email
     - **Common Name:** Your name or company name
     - **CA Email Address:** Leave EMPTY
     - **Request is:** Select **"Saved to disk"** (NOT "Emailed to the CA")
   - Click "Continue"
   - Save the `.certSigningRequest` file

2. **Generate Certificate on Apple Developer Portal:**
   - Go to https://developer.apple.com/account/resources/certificates/list
   - Click **+** to create a new certificate
   - Select **"Developer ID Application"** (for apps distributed outside Mac App Store)
   - Click "Continue"
   - Upload your `.certSigningRequest` file from step 1
   - Click "Continue"
   - Download the generated `.cer` file
   - **Double-click the `.cer` file** to install it in Keychain Access

3. **Verify Installation:**
   - Open **Keychain Access**
   - Select **"login"** keychain (left sidebar)
   - Select **"My Certificates"** category
   - Find `Developer ID Application: Your Name`
   - Click the **▶** disclosure triangle to expand it
   - **You MUST see a private key underneath** (with a key icon 🔑)
   - If no private key appears, DELETE this certificate and redo steps 1-2 on THIS Mac

### Step 2: Export Certificate for CI/CD

> **Note:** The certificate will only appear in Keychain Access after you complete Step 1 and double-click the downloaded `.cer` file to install it.

1. **Export as P12:**
   - Open **Keychain Access** app
   - In the left sidebar, select **"login"** keychain
   - Select **"My Certificates"** category
   - Find: `Developer ID Application: Your Name (TEAM_ID)` or `Developer ID Application`
   - **If you don't see it:** Make sure you completed Step 1 and double-clicked the downloaded `.cer` file
   - **IMPORTANT:** Click the **disclosure triangle (▶)** next to the certificate to expand it
   - You should see a private key underneath - if not, you need to redo Step 1
   - Right-click on the certificate (or the private key) → `Export "Developer ID Application..."`
   - The file will automatically be saved as `.p12` format (Personal Information Exchange)
   - Save as `Certificates.p12`
   - Set a strong password (save it - you'll need it for GitHub Secrets!)
   - Enter your Mac password to allow export

2. **Convert to Base64:**
   ```bash
   # In Terminal, navigate to where you saved Certificates.p12
   base64 -i Certificates.p12 | pbcopy
   ```
   This copies the encoded certificate to your clipboard.

### Step 3: Create App-˜Specific Password

1. Go to https://appleid.apple.com/account/manage
2. Sign in with your Apple ID
3. Under "Security" → "App-Specific Passwords" → Click **+**
4. Name it "GitHub Actions" or "Notarization"
5. **Copy and save the password** (you won't see it again!)

### Step 4: Get Your Team ID

1. Go to https://developer.apple.com/account
2. Click "Membership"
3. Copy your **Team ID** (10 characters, e.g., `AB1234CDEF`)

### Step 5: Configure GitHub Secrets

Go to your repository: **Settings** → **Secrets and variables** → **Actions**

Add these secrets:

| Secret Name | Value | Description |
|-------------|-------|-------------|
| `APPLE_CERTIFICATE_BASE64` | Base64 from Step 2 | Your signing certificate |
| `APPLE_CERTIFICATE_PASSWORD` | P12 password from Step 2 | Password for the certificate |
| `APPLE_TEAM_ID` | Team ID from Step 4 | Your Apple Developer Team ID |
| `APPLE_ID` | Your Apple ID email | Email for notarization |
| `APPLE_APP_SPECIFIC_PASSWORD` | Password from Step 3 | App-specific password |

### Step 6: Test the Workflow

1. **Trigger a release:**
   ```bash
   # Via workflow dispatch
   # Go to Actions → Release → Run workflow → Select bump type
   
   # Or via tag
   git tag v0.2.0
   git push origin v0.2.0
   ```

2. **Monitor the build:**
   - Look for the signing steps in the workflow logs
   - Notarization typically takes 5-15 minutes
   - Check for ✅ success messages

3. **Verify the signed app:**
   - Download the DMG from GitHub Release
   - Mount it and drag to Applications
   - Right-click → Open
   - Should open without Gatekeeper warnings!

### Troubleshooting

**"No identity found" error:**
- Make sure you selected "Developer ID Application" (not "Mac Development")
- Verify certificate is in the correct keychain

**Notarization timeout:**
- Increase timeout in workflow (default: 30 minutes)
- Check Apple system status: https://developer.apple.com/system-status/

**Gatekeeper still blocks:**
- Wait a few minutes for notarization to propagate
- Verify stapling: `xcrun stapler validate /path/to/app.dmg`
- Check signature: `spctl -a -vv /path/to/SEEGui.app`

---

## Windows Code Signing

### Prerequisites

1. **Code Signing Certificate** from a trusted CA
   - **Recommended:** Sectigo OV (~$200/year) or DigiCert EV (~$629/year)
   - **Purchase from:**
     - Sectigo: https://sectigo.com/ssl-certificates-tls/code-signing
     - DigiCert: https://www.digicert.com/signing/code-signing-certificates
     - SSL.com: https://www.ssl.com/code-signing/

### Certificate Types

**OV (Organization Validation)** - $200-500/year
- Good for most applications
- Validation takes 1-3 business days
- Requires business registration

**EV (Extended Validation)** - $400-700/year
- Builds SmartScreen reputation faster
- Comes on USB hardware token
- Validation takes 1-7 business days
- **Recommended for public releases**

### What You'll Need

For **Organization Validation:**
- Registered business entity
- DUNS number or business registration
- Business phone (publicly listed)
- Domain name validation
- Authorization letter

For **Individual:**
- Government-issued ID
- Fewer CAs offer this option
- ~$100-200/year

### Setup Steps (Once You Have Certificate)

1. **Export certificate as PFX:**
   - Include private key
   - Set a strong password

2. **Convert to Base64:**
   ```powershell
   # Windows PowerShell
   $bytes = [System.IO.File]::ReadAllBytes("certificate.pfx")
   [Convert]::ToBase64String($bytes) | clip
   ```

3. **Add GitHub Secrets:**

| Secret Name | Value | Description |
|-------------|-------|-------------|
| `WINDOWS_CERTIFICATE_BASE64` | Base64 from step 2 | Your signing certificate |
| `WINDOWS_CERTIFICATE_PASSWORD` | PFX password | Password for certificate |

4. **Update workflow** (future work):
   - Add certificate import step
   - Sign MSI with `signtool.exe`
   - Sign CLI binary

### Cost Comparison

| CA | OV Price | EV Price | Reputation Build |
|----|----------|----------|------------------|
| Sectigo | ~$200/year | ~$400/year | Moderate |
| DigiCert | ~$474/year | ~$629/year | Fast |
| SSL.com | ~$200/year | ~$400/year | Moderate |
| GlobalSign | ~$300/year | ~$600/year | Moderate |

---

## Release Without Signing

The workflows are designed to work **without** signing certificates:

- If no `APPLE_CERTIFICATE_BASE64` is found, macOS builds will be unsigned
- Users will see Gatekeeper warnings (can bypass with `xattr -cr`)
- Windows users will see SmartScreen warnings (can bypass with "Run anyway")

This allows you to:
1. Release immediately while waiting for certificates
2. Build SmartScreen reputation organically
3. Add signing later when ready

---

## Annual Costs Summary

| Service | Cost | Required |
|---------|------|----------|
| Apple Developer Program | $99/year | For macOS signing |
| Windows Code Signing (OV) | $200-500/year | For Windows signing |
| Windows Code Signing (EV) | $400-700/year | For Windows signing (faster reputation) |

**Total for both platforms:** ~$300-800/year depending on certificate types.

---

## Questions?

- **Apple Code Signing:** https://developer.apple.com/support/code-signing/
- **Notarization:** https://developer.apple.com/documentation/security/notarizing_macos_software_before_distribution
- **Windows Authenticode:** https://docs.microsoft.com/en-us/windows-hardware/drivers/install/authenticode

