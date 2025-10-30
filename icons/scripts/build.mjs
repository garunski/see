import fs from "fs/promises";
import path from "path";
import { fileURLToPath } from "url";
import sharp from "sharp";
import { createICNS, createICO, BICUBIC } from "@ctjs/png2icons";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const rootDir = path.join(__dirname, "..");
const svgPath = path.join(rootDir, "branding", "s_e_e.svg");
const distDir = path.join(rootDir, "dist");

// Size definitions
const SIZES = {
  png: [16, 22, 24, 32, 48, 64, 96, 128, 256, 512, 1024],
  hicolor: [16, 22, 24, 32, 48, 64, 128, 256, 512],
  favicon: [16, 32],
  og: 512,
};

async function validateSVG() {
  console.log("📋 Validating SVG...");
  const svgBuffer = await fs.readFile(svgPath);
  const metadata = await sharp(svgBuffer).metadata();

  if (metadata.width !== metadata.height) {
    throw new Error(
      `SVG must be square. Got ${metadata.width}x${metadata.height}`
    );
  }

  console.log(`✓ SVG validated: ${metadata.width}x${metadata.height}`);
  return svgBuffer;
}

async function generatePNGs(svgBuffer) {
  console.log("🎨 Generating PNGs...");

  await Promise.all(
    SIZES.png.map(async (size) => {
      const outputDir = path.join(distDir, "png", size.toString());
      await fs.mkdir(outputDir, { recursive: true });

      await sharp(svgBuffer)
        .resize(size, size, {
          fit: "contain",
          background: { r: 0, g: 0, b: 0, alpha: 0 },
        })
        .png({
          compressionLevel: 9,
          adaptiveFiltering: true,
        })
        .toFile(path.join(outputDir, "see.png"));

      console.log(`  ✓ Generated ${size}x${size} PNG`);
    })
  );
}

async function assembleICO() {
  console.log("🪟 Assembling Windows ICO...");

  // Use the 1024px PNG as input for png2icons
  const inputPngPath = path.join(distDir, "png", "1024", "see.png");
  const inputBuffer = await fs.readFile(inputPngPath);

  // Create ICO with mix of BMP and PNG for Windows executables
  // usePNG=false, forWinExe=true means small icons as BMP, large as PNG
  const icoBuffer = createICO(inputBuffer, BICUBIC, 0, false, true);

  if (!icoBuffer) {
    throw new Error("Failed to create ICO file");
  }

  const windowsDir = path.join(distDir, "windows");
  await fs.mkdir(windowsDir, { recursive: true });
  await fs.writeFile(path.join(windowsDir, "see.ico"), icoBuffer);

  console.log("  ✓ Created see.ico");
}

async function assembleICNS() {
  console.log("🍎 Assembling macOS ICNS...");

  // Use the 1024px PNG as input for png2icons
  const inputPngPath = path.join(distDir, "png", "1024", "see.png");
  const inputBuffer = await fs.readFile(inputPngPath);

  // Create ICNS with bicubic interpolation and no color reduction
  const icnsBuffer = createICNS(inputBuffer, BICUBIC, 0);

  if (!icnsBuffer) {
    throw new Error("Failed to create ICNS file");
  }

  const macosDir = path.join(distDir, "macos");
  await fs.mkdir(macosDir, { recursive: true });
  await fs.writeFile(path.join(macosDir, "see.icns"), icnsBuffer);

  console.log("  ✓ Created see.icns");
}

async function buildHicolor() {
  console.log("🐧 Building Linux hicolor layout...");

  for (const size of SIZES.hicolor) {
    const targetDir = path.join(
      distDir,
      "linux",
      "hicolor",
      `${size}x${size}`,
      "apps"
    );
    await fs.mkdir(targetDir, { recursive: true });

    const sourcePath = path.join(distDir, "png", size.toString(), "see.png");
    const targetPath = path.join(targetDir, "see.png");

    try {
      // Try hardlink first (saves space)
      await fs.link(sourcePath, targetPath);
      console.log(`  ✓ Hardlinked ${size}x${size}`);
    } catch (err) {
      // Fallback to copy if hardlink fails (e.g., on Windows or cross-device)
      await fs.copyFile(sourcePath, targetPath);
      console.log(`  ✓ Copied ${size}x${size}`);
    }
  }
}

async function buildFavicons() {
  console.log("🌐 Building web favicons...");

  const webDir = path.join(distDir, "web");
  await fs.mkdir(webDir, { recursive: true });

  // Copy favicon PNGs
  for (const size of SIZES.favicon) {
    const source = path.join(distDir, "png", size.toString(), "see.png");
    const dest = path.join(webDir, `favicon-${size}.png`);
    await fs.copyFile(source, dest);
    console.log(`  ✓ Created favicon-${size}.png`);
  }

  // Create favicon.ico using png2icons
  const inputPngPath = path.join(distDir, "png", "256", "see.png");
  const inputBuffer = await fs.readFile(inputPngPath);
  
  // Create ICO with PNG format for web (smaller file size)
  const faviconBuffer = createICO(inputBuffer, BICUBIC, 0, true, false);
  
  if (!faviconBuffer) {
    throw new Error("Failed to create favicon.ico");
  }
  
  await fs.writeFile(path.join(webDir, "favicon.ico"), faviconBuffer);
  console.log("  ✓ Created favicon.ico");
}

async function buildOGImage() {
  console.log("📱 Building OG image...");

  const iconPath = path.join(distDir, "png", SIZES.og.toString(), "see.png");

  await sharp({
    create: {
      width: 1200,
      height: 630,
      channels: 4,
      background: { r: 255, g: 255, b: 255, alpha: 1 },
    },
  })
    .composite([
      {
        input: iconPath,
        gravity: "center",
      },
    ])
    .png()
    .toFile(path.join(distDir, "web", "og-1200x630.png"));

  console.log("  ✓ Created og-1200x630.png");
}

async function main() {
  try {
    console.log("🚀 Starting icon generation\n");

    // Clean dist directory
    try {
      await fs.rm(distDir, { recursive: true, force: true });
    } catch (err) {
      // Ignore if doesn't exist
    }

    const svgBuffer = await validateSVG();
    await generatePNGs(svgBuffer);
    await assembleICO();
    await assembleICNS();
    await buildHicolor();
    await buildFavicons();
    await buildOGImage();

    console.log("\n✅ Icon generation complete!");
  } catch (error) {
    console.error("\n❌ Error:", error.message);
    process.exit(1);
  }
}

main();
