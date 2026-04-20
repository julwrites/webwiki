import re

with open("frontend/src/components/bottom_bar.rs", "r") as f:
    content = f.read()

content = re.sub(
    r'<div class="bottom-bar-search-trigger" onclick=\{move \|_\| on_search\.emit\(\)\}>\s*<IconSearch />\s*<span>\{"Search files\.\.\."\}</span>\s*</div>',
    r'<button class="bottom-bar-search-trigger" onclick={move |_| on_search.emit(())} aria-label="Search files" title="Search files">\n                <IconSearch />\n                <span>{"Search files..."}</span>\n            </button>',
    content
)

content = re.sub(
    r'<button onclick=\{\s*let on_pull = props\.on_pull\.clone\(\);\s*let close = close_git_menu\.clone\(\);\s*move \|_\| \{ on_pull\.emit\(\(\)\); close\.emit\(\(\)\); \}\s*\}>',
    r'<button onclick={\n                                let on_pull = props.on_pull.clone();\n                                let close = close_git_menu.clone();\n                                move |_| { on_pull.emit(()); close.emit(()); }\n                            } title="Pull" aria-label="Pull">',
    content
)

content = re.sub(
    r'<button onclick=\{\s*let on_commit = props\.on_commit\.clone\(\);\s*let close = close_git_menu\.clone\(\);\s*move \|_\| \{ on_commit\.emit\(\(\)\); close\.emit\(\(\)\); \}\s*\}>',
    r'<button onclick={\n                                let on_commit = props.on_commit.clone();\n                                let close = close_git_menu.clone();\n                                move |_| { on_commit.emit(()); close.emit(()); }\n                            } title="Commit" aria-label="Commit">',
    content
)

content = re.sub(
    r'<button onclick=\{\s*let on_push = props\.on_push\.clone\(\);\s*let close = close_git_menu\.clone\(\);\s*move \|_\| \{ on_push\.emit\(\(\)\); close\.emit\(\(\)\); \}\s*\}>',
    r'<button onclick={\n                                let on_push = props.on_push.clone();\n                                let close = close_git_menu.clone();\n                                move |_| { on_push.emit(()); close.emit(()); }\n                            } title="Push" aria-label="Push">',
    content
)

with open("frontend/src/components/bottom_bar.rs", "w") as f:
    f.write(content)
