import sys, json, os
sys.path.insert(0, '/home/user/age-of-agents/assets/ui/tools')
import falcall
S = '/tmp/claude-0/-home-user-age-of-agents/16660f84-9487-5e5a-b78c-8d6f01e25d5e/scratchpad/spike/'
name, mode = sys.argv[1], sys.argv[2]
img = falcall.data_uri(S + (f'{name}_padded.png' if mode == 'img' else f'{name}.png'))
if mode == '3d':
    out = falcall.run('fal-ai/trellis', {'image_url': img, 'mesh_simplify': 0.9, 'texture_size': 512}, 0.35, f'shadow-spike-3d-{name}')
    json.dump(out, open(S + f'{name}.trellis.json', 'w'))
    falcall.download(out['model_mesh']['url'], S + f'{name}.glb')
else:
    prompt = ('Keep this isometric painted building exactly as it is. Add only the shadow it casts on flat ground: '
              'the sun is high to the upper left, so the shadow lies on the ground to the right of and behind the building, '
              'following the building silhouette, as a flat soft dark grey-blue shape. No other changes, no ground texture, '
              'keep the building at exactly the same position and size, and keep the plain white background outside the shadow.')
    out = falcall.run('fal-ai/nano-banana/edit', {'prompt': prompt, 'image_urls': [img], 'num_images': 1, 'output_format': 'png'}, 0.04, f'shadow-spike-img-{name}')
    json.dump(out, open(S + f'{name}.shadow.json', 'w'))
    falcall.download(out['images'][0]['url'], S + f'{name}.shadow.png')
print('done', name, mode)
