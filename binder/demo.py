import re

from IPython import get_ipython

MULTILINE_PATTERN = re.compile(r"('\(.*?\)')", re.DOTALL)

def inline_match(match):
    return ' '.join(match.group(1).split())

# Replace multiline command argument strings with single-line version
def inline_multiline_command(lines):
    return [line + '\n' for line in
        MULTILINE_PATTERN.sub(inline_match, '\n'.join(lines)).split('\n')
    if len(line) > 0]

get_ipython().input_transformers_cleanup.append(inline_multiline_command)
