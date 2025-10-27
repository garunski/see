import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const heroiconsDir = path.join(__dirname, 'node_modules', 'heroicons');
const outputDir = path.join(__dirname, '..', 'gui', 'assets', 'icons');

// Icon mapping: our name -> heroicons file name
const iconMap = {
  home: 'home',
  upload: 'arrow-up-tray',
  workflows: 'squares-2x2',
  history: 'clock',
  prompts: 'chat-bubble-left-right',
  settings: 'cog-6-tooth',
  plus: 'plus',
  x: 'x-mark',
  chevron_left: 'chevron-left',
  chevron_right: 'chevron-right',
  arrow_left: 'arrow-left',
  trash: 'trash',
  exclamation_circle: 'exclamation-circle',
  play: 'play',
  stop: 'stop',
  bars_3: 'bars-3',
  copy: 'document-duplicate',
  save: 'arrow-down-on-square'
};

// Ensure output directory exists
if (!fs.existsSync(outputDir)) {
  fs.mkdirSync(outputDir, { recursive: true });
}

console.log('Copying Heroicons...');

// Copy outline variant only
Object.entries(iconMap).forEach(([ourName, heroiconName]) => {
  // Copy outline variant
  const outlineSrc = path.join(heroiconsDir, '24', 'outline', `${heroiconName}.svg`);
  const outlineDest = path.join(outputDir, `${ourName}-outline.svg`);
  if (fs.existsSync(outlineSrc)) {
    fs.copyFileSync(outlineSrc, outlineDest);
    console.log(`✓ Copied ${ourName}-outline.svg`);
  } else {
    console.log(`✗ Missing: ${outlineSrc}`);
  }
});

console.log('Icon copying complete!');
