// Words and placeholder vector icons for the HUD. Icons are 24-unit SVG bodies;
// generated art can replace any of them without touching layout code.
export const RESOURCES = [
  ['wood', 'Wood', '<rect x="2" y="8" width="18" height="8" rx="4" fill="#b07a42" stroke="#3a2210"/><ellipse cx="19" cy="12" rx="3" ry="4" fill="#f0cf96" stroke="#3a2210"/><circle cx="19" cy="12" r="1.3" fill="#b07a42"/>'],
  ['food', 'Food', '<circle cx="8.5" cy="14" r="4.2" fill="#e0303f" stroke="#4a0c12"/><circle cx="15.5" cy="15" r="3.8" fill="#c21f30" stroke="#4a0c12"/><circle cx="12" cy="8.5" r="3.6" fill="#f0485a" stroke="#4a0c12"/><path d="M12 5c1-2 3-3 5-3" stroke="#3f7a2a" stroke-width="1.8" fill="none"/>'],
  ['stone', 'Stone', '<path d="M3 18 6 8l8-4 7 6 1 8z" fill="#d6d0c4" stroke="#3a3228"/><path d="M6 8l8-4 1 7z" fill="#f2eee6"/>'],
  ['gold', 'Gold', '<path d="M12 2 20 12 12 22 4 12z" fill="#ffcf2e" stroke="#6a4a00"/><path d="M12 2 20 12h-8z" fill="#fff2a0"/>'],
  ['iron', 'Iron', '<path d="M4 17h16l-2.5-7h-11z" fill="#8a8fa0" stroke="#2a2e38"/><path d="M6.5 10h11L16 7.5H8z" fill="#c4c9d6" stroke="#2a2e38"/>'],
  ['clay', 'Clay', '<path d="M8 4h8l-1 3c3 1 4.5 4 3.5 7-1 3.5-4 6-6.5 6S7 17.5 6 14c-1-3 .5-6 3.5-7z" fill="#e2784c" stroke="#5a1e0c"/>'],
  ['fiber', 'Fiber', '<path d="M12 22V6" stroke="#6f8a2a" stroke-width="1.8"/><ellipse cx="12" cy="5" rx="2.2" ry="3.2" fill="#f2df6a" stroke="#6a5a10"/><ellipse cx="8.5" cy="10.5" rx="1.8" ry="2.8" fill="#f2df6a" stroke="#6a5a10" transform="rotate(-30 8.5 10.5)"/><ellipse cx="15.5" cy="10.5" rx="1.8" ry="2.8" fill="#f2df6a" stroke="#6a5a10" transform="rotate(30 15.5 10.5)"/>']
];
export const ICONS = {
  build: '<path d="M4 20V11l8-6 8 6v9z" fill="#e0452e" stroke="#2c1b0e" stroke-width="1.5"/><path d="M7 20v-7h10v7" fill="#fbefd2" stroke="#2c1b0e" stroke-width="1.5"/><path d="M10.5 20v-4h3v4" fill="#6b4528"/>',
  cancel: '<path d="M6 6l12 12M18 6 6 18" stroke="#7d1f17" stroke-width="3.5" stroke-linecap="round"/>',
  villager: '<circle cx="12" cy="7" r="3.6" fill="#f2c9a0" stroke="#2c1b0e" stroke-width="1.3"/><path d="M5 21v-4.5C5 13 8 11.5 12 11.5s7 1.5 7 5V21z" fill="#5b8e7d" stroke="#2c1b0e" stroke-width="1.3"/><path d="M8.5 12.2h7l-1 1.8h-5z" fill="#2f6fe0"/>',
  forestry: '<path d="M12 3 6 13h3l-3 5h12l-3-5h3z" fill="#4f9a5a" stroke="#1e3a20" stroke-width="1.3"/><rect x="11" y="18" width="2" height="4" fill="#6b4528"/>',
  agriculture: '<path d="M12 22V8" stroke="#6f8a2a" stroke-width="1.8"/><ellipse cx="12" cy="6" rx="2.4" ry="3.6" fill="#f2c84a" stroke="#6a4a00"/><ellipse cx="8.5" cy="12" rx="2" ry="3" fill="#f2c84a" stroke="#6a4a00" transform="rotate(-35 8.5 12)"/><ellipse cx="15.5" cy="12" rx="2" ry="3" fill="#f2c84a" stroke="#6a4a00" transform="rotate(35 15.5 12)"/>',
  masonry: '<rect x="3" y="14" width="8" height="5" fill="#d6d0c4" stroke="#3a3228"/><rect x="13" y="14" width="8" height="5" fill="#d6d0c4" stroke="#3a3228"/><rect x="8" y="8" width="8" height="5" fill="#e2784c" stroke="#5a1e0c"/>',
  mining: '<path d="M4 9c4-5 12-5 16 0" stroke="#5a5f6e" stroke-width="2.6" fill="none" stroke-linecap="round"/><path d="M12 6v15" stroke="#8a5a30" stroke-width="2.4" stroke-linecap="round"/><path d="M16 17l3 3-3 2-2-3z" fill="#ffcf2e" stroke="#6a4a00"/>',
  textiles: '<ellipse cx="12" cy="12" rx="6" ry="8" fill="#f2df6a" stroke="#6a5a10" stroke-width="1.3"/><path d="M6.5 9h11M6 12h12M6.5 15h11" stroke="#b8a03a"/><path d="M18 18l3 3" stroke="#7a7f8c" stroke-width="1.6"/>',
  townCenter: '<path d="M3 21V12l9-7 9 7v9z" fill="#e0452e" stroke="#2c1b0e" stroke-width="1.3"/><rect x="6" y="13" width="12" height="8" fill="#fbefd2" stroke="#2c1b0e" stroke-width="1.3"/><rect x="10.5" y="16" width="3" height="5" fill="#6b4528"/><path d="M12 5V1.5l4 1.5-4 1.5" fill="#2f6fe0" stroke="#2c1b0e" stroke-width=".8"/>',
  group: '<circle cx="8" cy="8" r="3" fill="#f2c9a0" stroke="#2c1b0e"/><circle cx="16" cy="8" r="3" fill="#d8a47a" stroke="#2c1b0e"/><path d="M2 20v-3.5c0-3 2.5-4.5 6-4.5s6 1.5 6 4.5V20z" fill="#5b8e7d" stroke="#2c1b0e"/><path d="M10 20v-3.5c0-3 2.5-4.5 6-4.5s6 1.5 6 4.5V20z" fill="#c8553d" stroke="#2c1b0e"/>',
  scroll: '<path d="M6 4h11a2 2 0 0 1 2 2v12a2 2 0 0 1-2 2H7" fill="#fbefd2" stroke="#6b4528" stroke-width="1.4"/><path d="M6 4a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h1V6a2 2 0 0 0-1-2z" fill="#e2c992" stroke="#6b4528" stroke-width="1.4"/><path d="M10 9h6M10 12h6M10 15h4" stroke="#8a6524"/>'
};
export const TECHNOLOGIES = {
  forestry: ['Forestry', 'Wood +20%'],
  agriculture: ['Agriculture', 'Food +20%'],
  masonry: ['Masonry', 'Stone and clay +20%'],
  mining: ['Mining', 'Gold and iron +20%'],
  textiles: ['Textiles', 'Fiber +20%']
};
export const PREREQUISITE = { mining: 'masonry', textiles: 'agriculture' };
export const FRIENDLY_ERRORS = {
  'unit is busy': 'That villager is busy with its current task.',
  'destination cell is occupied': 'Something already stands there.',
  'target is unreachable': 'No path leads there.',
  'build site is blocked or outside the world': 'The town center needs a clear 2×2 site.',
  'insufficient wood': 'You need 20 wood to build.',
  'insufficient food': 'You need 50 food to train a villager.'
};
export const ACTIVITY_TEXT = {
  idle: 'Awaiting orders', move: 'Walking', build: 'Building',
  to_resource: 'Heading out to gather', gathering: 'Gathering', returning: 'Carrying goods home', depositing: 'Unloading'
};

// Generated illustrations under assets/ui (see assets/ui/manifest.json), keyed
// like the placeholders above.
export const ART = {
  wood: 'icons/resource_wood.png', food: 'icons/resource_food.png', stone: 'icons/resource_stone.png',
  gold: 'icons/resource_gold.png', iron: 'icons/resource_iron.png', clay: 'icons/resource_clay.png',
  fiber: 'icons/resource_fiber.png',
  build: 'icons/command_build.png', cancel: 'icons/command_cancel.png', train: 'icons/command_train.png',
  forestry: 'icons/tech_forestry.png', agriculture: 'icons/tech_agriculture.png', masonry: 'icons/tech_masonry.png',
  mining: 'icons/tech_mining.png', textiles: 'icons/tech_textiles.png',
  portrait_villager: 'icons/portrait_villager.png', portrait_group: 'icons/portrait_group.png',
  portrait_townCenter: 'icons/portrait_towncenter.png'
};
