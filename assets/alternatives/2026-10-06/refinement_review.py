"""Four-subject bounded refinement review; no generation or runtime edits."""
from correction_review import sheet,BASE,source

def latest(name):
 p=BASE/'11d-map-shield-targeted-retries'/name/'cutout.png'
 return p if p.exists() else BASE/'11b-painted-cel-subjects'/name/'cutout.png'

sheet('painted-cel-refinement-comparison.jpg','PAINTED CEL REFINEMENT | current / preceding attempt / new UNAPPROVED + original villager',[
 ('Steel',source('resource_steel'),BASE/'10-cel-calibration/resource_steel/cutout.png',BASE/'11a-painted-cel-steel/resource_steel/cutout.png','icon','darker painted option A; option B separately shown, neither approved'),
 ('Wolf',BASE/'wolf-current-crop.png',BASE/'10-cel-calibration/wolf_idle/cutout.png',(BASE/'11e-wolf-cool-coat-retry/wolf_idle/cutout.png' if (BASE/'11e-wolf-cool-coat-retry/wolf_idle/cutout.png').exists() else BASE/'11b-painted-cel-subjects/wolf_idle/cutout.png'),'sprite','right-facing geometry retained; COOL RECOLOR FAILED, still tan'),
 ('Explore',source('command_explore'),BASE/'09b-era-explore-calibration/command_explore/cutout.png',latest('command_explore'),'icon','runtime compass ERA REJECTED; simpler map, contour/shadow still unresolved'),
 ('Military',source('category_military'),BASE/'07-targeted-repairs/category_military/cutout.png',latest('category_military'),'icon','quieter cel face; spear correctly behind; promising but unapproved')])

sheet("painted-steel-options-comparison.jpg","STEEL OPTIONS | runtime / darker painted A / paler painted B; neither assumed better",[("Steel",source("resource_steel"),BASE/"11a-painted-cel-steel/resource_steel/cutout.png",BASE/"11c-steel-targeted-retry/resource_steel/cutout.png","icon","A richer but more marks; B quieter/paler and very close to retained source")],preceding_label="Option A darker painted",latest_label="Option B lighter painted",small_label="Small-scale runtime / A / B")
