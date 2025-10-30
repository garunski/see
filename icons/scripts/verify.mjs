import fs from "fs/promises";
import path from "path";
import { fileURLToPath } from "url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const rootDir = path.join(__dirname, "..");
const distDir = path.join(rootDir, "dist");

const SIZES = {
  png: [16, 22, 24, 32, 48, 64, 96, 128, 256, 512, 1024],
  hicolor: [16, 22, 24, 32, 48, 64, 128, 256, 512],
  favicon: [16, 32],
};

async function checkFile(filePath, description) {
  try {
    await fs.access(filePath);
    console.log(`✓ ${description}`);
    return true;
  } catch (err) {
    console.error(`✗ Missing: ${description} (${filePath})`);
    return false;
  }
}

async function verify() {
  console.log("🔍 Verifying generated icons\n");

  let allPassed = true;

  // Check PNG files
  console.log("PNG Files:");
  for (const size of SIZES.png) {
    const filePath = path.join(distDir, "png", size.toString(), "see.png");
    const passed = await checkFile(filePath, `${size}x${size} PNG`);
    allPassed = allPassed && passed;
  }

  // Check Windows ICO
  console.log("\nWindows:");
  const icoPath = path.join(distDir, "windows", "see.ico");
  const icoPassed = await checkFile(icoPath, "see.ico");
  allPassed = allPassed && icoPassed;

  // Check macOS ICNS
  console.log("\nmacOS:");
  const icnsPath = path.join(distDir, "macos", "see.icns");
  const icnsPassed = await checkFile(icnsPath, "see.icns");
  allPassed = allPassed && icnsPassed;

  // Check Linux hicolor
  console.log("\nLinux Hicolor:");
  for (const size of SIZES.hicolor) {
    const filePath = path.join(
      distDir,
      "linux",
      "hicolor",
      `${size}x${size}`,
      "apps",
      "see.png"
    );
    const passed = await checkFile(
      filePath,
      `hicolor/${size}x${size}/apps/see.png`
    );
    allPassed = allPassed && passed;
  }

  // Check web assets
  console.log("\nWeb Assets:");
  const faviconIcoPath = path.join(distDir, "web", "favicon.ico");
  const faviconIcoPassed = await checkFile(faviconIcoPath, "favicon.ico");
  allPassed = allPassed && faviconIcoPassed;

  for (const size of SIZES.favicon) {
    const filePath = path.join(distDir, "web", `favicon-${size}.png`);
    const passed = await checkFile(filePath, `favicon-${size}.png`);
    allPassed = allPassed && passed;
  }

  const ogPath = path.join(distDir, "web", "og-1200x630.png");
  const ogPassed = await checkFile(ogPath, "og-1200x630.png");
  allPassed = allPassed && ogPassed;

  console.log("\n" + "=".repeat(50));
  if (allPassed) {
    console.log("✅ All icon files verified successfully!");
    process.exit(0);
  } else {
    console.log("❌ Some icon files are missing");
    process.exit(1);
  }
}

verify();
