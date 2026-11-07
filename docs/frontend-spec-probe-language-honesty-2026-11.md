# Frontend spec — Probe-language honesty (L1)

**Date (UTC):** 2026-11-06  
**Status:** Implemented  
**Depends on:** [probe-language-honesty-design.md](../specs/backend/probe-language-honesty-design.md)  
**Surfaces:** Overview Quick hero · Advanced diagnosis / path / stability

## Purpose

zh-Hans users are not left guessing why chrome is Chinese but probe narratives are English.

## Acceptance

- [x] zh-Hans + shaped diagnosis → Quick hero uses i18n summary key when available  
- [x] zh-Hans + English hero text → English measurement note visible  
- [x] zh-Hans Advanced diagnosis/path/stability show English-measurement badge/note  
- [x] `en` locale shows no English-measurement badge  
