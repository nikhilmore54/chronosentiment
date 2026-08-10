import re

with open('apps/ultracrew-pilot-portal/src/App.js', 'r') as f:
    content = f.read()

# 1. Add imports
import_str = """import GanttChart from './components/GanttChart';
import DisruptionPanel from './components/DisruptionPanel';
"""
content = content.replace("import React, { useState, useEffect, useCallback } from 'react';", "import React, { useState, useEffect, useCallback } from 'react';\n" + import_str)

# 2. Extract DisruptionPanel IIFE
disruption_iife_start = "{/* ── Disruption Simulation Panel ── */}"
# We will use regex to find the IIFE block
import ast
# actually just replacing string is easier if we find the start and end precisely.
