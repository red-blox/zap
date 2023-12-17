---
# https://vitepress.dev/reference/default-theme-home-page
layout: home

hero:
  name: "Zap"
  tagline: A blazingly fast networking solution for Roblox.
  actions:
    - theme: brand
      text: Get Started
      link: /install
    - theme: alt
      text: Try it out
      link: /playground

features:
  - title: Type Safety
    icon: 🔐
    details: Zap generates a fully type-safe API for your network configuration. This means full intellisense support with autocomplete and type checking.
  - title: Performance
    icon: ⚡
    details: |
      Zap packs all data into buffers to send over the network.
      This has the obvious benefits of reduced bandwidth usage, however the serialization and deserialization process is typically quite slow.
      Zap generates code for your specific types which makes this process blazingly fast.
  - title: Complex Types
    icon: 🔎
    details: While buffers may only support a small number of types, zap has complex type support.
---

