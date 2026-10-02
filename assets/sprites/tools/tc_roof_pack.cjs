// Pack reviewed image-tool renders without moving the existing sprite footprint.
// Requires sharp. Usage: node tc_roof_pack.cjs complete.png working.png atlas.png
const sharp = require('sharp');
const fs = require('node:fs/promises');

async function bounds(image) {
  const { data, info } = await sharp(image).ensureAlpha().raw()
    .toBuffer({ resolveWithObject: true });
  let left = info.width, top = info.height, right = 0, bottom = 0;
  for (let y = 0; y < info.height; y++) {
    for (let x = 0; x < info.width; x++) {
      if (data[(y * info.width + x) * 4 + 3] <= 40) continue;
      left = Math.min(left, x); top = Math.min(top, y);
      right = Math.max(right, x + 1); bottom = Math.max(bottom, y + 1);
    }
  }
  if (right <= left || bottom <= top) throw new Error('Empty sprite');
  return { left, top, width: right - left, height: bottom - top };
}

async function cell(image, target) {
  const crop = await bounds(image);
  const fitted = await sharp(image).extract(crop)
    .resize(target.width, target.height, { fit: 'fill' }).png().toBuffer();
  return sharp({ create: { width: 512, height: 512, channels: 4,
    background: { r: 0, g: 0, b: 0, alpha: 0 } } })
    .composite([{ input: fitted, left: target.left, top: target.top }])
    .png().toBuffer();
}

async function main() {
  const [complete, working, atlas] = process.argv.slice(2);
  if (!complete || !working || !atlas) {
    throw new Error('Usage: tc_roof_pack.cjs complete.png working.png atlas.png');
  }
  const original = await fs.readFile(atlas);
  const metadata = await sharp(original).metadata();
  if (metadata.width !== 2560 || metadata.height !== 512 || !metadata.hasAlpha) {
    throw new Error('Expected the existing 2560×512 RGBA atlas');
  }
  const oldComplete = await sharp(original)
    .extract({ left: 1536, top: 0, width: 512, height: 512 }).png().toBuffer();
  const target = await bounds(oldComplete);
  const idleCell = await cell(complete, target);
  const workingCell = await cell(working, target);
  // Always reuse the exact complete-state roof and pennant in the working frame.
  const idle = await sharp(idleCell).ensureAlpha().raw().toBuffer();
  const busy = await sharp(workingCell).ensureAlpha().raw().toBuffer();
  idle.copy(busy, 0, 0, 260 * 512 * 4);
  const pixels = await sharp(original).ensureAlpha().raw().toBuffer();
  for (let y = 0; y < 512; y++) {
    idle.copy(pixels, (y * 2560 + 1536) * 4, y * 512 * 4, (y + 1) * 512 * 4);
    busy.copy(pixels, (y * 2560 + 2048) * 4, y * 512 * 4, (y + 1) * 512 * 4);
  }
  const result = await sharp(pixels, { raw: { width: 2560, height: 512, channels: 4 } })
    .png().toBuffer();
  await fs.writeFile(atlas, result);
  console.log('Preserved construction cells and manifest; fitted finished cells:', target);
}

main().catch(error => { console.error(error.message); process.exitCode = 1; });
