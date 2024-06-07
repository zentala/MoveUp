import path from 'path';
import fs from 'fs';
import sharp from 'sharp';
import { nativeImage, NativeImage } from 'electron';

let trayIconWhite: NativeImage;
let trayIconBlack: NativeImage;

export async function generateIcons() {
  const trayIconPath = path.join(__dirname, 'icon.svg');
  const trayIconSVG = fs.readFileSync(trayIconPath, 'utf8');

  // Create white and black icons
  const whiteIconSVG = trayIconSVG.replace(/<path/g, '<path fill="#FFFFFF" stroke="#FFFFFF"');
  const blackIconSVG = trayIconSVG.replace(/<path/g, '<path fill="#000000" stroke="#000000"');

  const whiteIconPNG = await sharp(Buffer.from(whiteIconSVG)).png().resize(16, 16).toBuffer();
  const blackIconPNG = await sharp(Buffer.from(blackIconSVG)).png().resize(16, 16).toBuffer();

  trayIconWhite = nativeImage.createFromBuffer(whiteIconPNG);
  trayIconBlack = nativeImage.createFromBuffer(blackIconPNG);
}

export function getIcon(isDarkMode: boolean): NativeImage {
  return isDarkMode ? trayIconWhite : trayIconBlack;
}
