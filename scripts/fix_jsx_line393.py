#!/usr/bin/env python3
"""
Fix JSX syntax error at line 393 of App.js.
The text node 'Junior Dispatcher (<2 years)' contains a raw U+003C (<)
which JSX cannot parse in text content. Replace it with the HTML entity &lt;
"""
import sys

path = 'apps/ultracrew-pilot-portal/src/App.js'

with open(path, 'rb') as f:
    data = f.read()

# The bad line as raw bytes (real < characters, U+003C = 0x3C)
# value attr has (<2 years) — raw < is OK inside quoted attr
# text content has (<2 years) — raw < is INVALID in JSX text node
bad  = b'              <option value="Junior Dispatcher (<2 years)">Junior Dispatcher (<2 years)</option>'
# Fixed: text content uses &lt; entity; value attr keeps raw < (valid in quoted string)
good = b'              <option value="Junior Dispatcher (<2 years)">Junior Dispatcher (&lt;2 years)</option>'

print(f'bad  bytes: {bad!r}')
print(f'good bytes: {good!r}')
print(f'bad  in file: {bad in data}')

if bad in data:
    data2 = data.replace(bad, good, 1)
    with open(path, 'wb') as f:
        f.write(data2)
    print('SUCCESS: replaced bad line with fixed line')
    # Verify
    with open(path, 'rb') as f:
        data3 = f.read()
    lines = data3.split(b'\n')
    line393 = lines[392]
    print(f'Line 393 after fix: {line393!r}')
    # Check for raw < in text content position
    idx = line393.find(b'>Junior Dispatcher (')
    if idx >= 0:
        snippet = line393[idx:idx+30]
        print(f'Text content snippet: {snippet!r}')
        if b'<' in snippet:
            print('ERROR: still has raw < in text content!')
            sys.exit(1)
        else:
            print('OK: no raw < in text content')
else:
    print('ERROR: bad line not found in file')
    # Show what line 393 actually looks like
    lines = data.split(b'\n')
    line393 = lines[392]
    print(f'Line 393 actual: {line393!r}')
    sys.exit(1)