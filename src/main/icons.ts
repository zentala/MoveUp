import { nativeImage, NativeImage } from 'electron';
import iconSVG from '../icon.svg?raw';

let trayIconWhite: NativeImage;
let trayIconBlack: NativeImage;

export async function generateIcons() {
  // Create white and black icons
  const whiteIconSVG = iconSVG.replace(/<path/g, '<path fill="#FFFFFF" stroke="#FFFFFF"');
  const blackIconSVG = iconSVG.replace(/<path/g, '<path fill="#000000" stroke="#000000"');

  const asImage = (svg: string): NativeImage =>
    nativeImage.createFromDataURL(`data:image/svg+xml;base64,${Buffer.from(svg).toString('base64')}`);

  trayIconWhite = asImage(whiteIconSVG);
  trayIconBlack = asImage(blackIconSVG);
}

export function getIcon(isDarkMode: boolean): NativeImage {
  return isDarkMode ? trayIconWhite : trayIconBlack;
}
