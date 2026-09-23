export function createProviderForm(prompt, preset = null) {
  return {
    id: null, name: preset?.name || '', kind: preset?.kind || 'openai_compatible',
    baseUrl: preset?.baseUrl || '', model: preset?.model || '',
    apiKey: '', hasApiKey: false, enabled: true,
    autoApplyTags: false, autoMarkReviewed: false, minConfidence: 0.75, maxTags: 12,
    language: 'zh-CN', systemPrompt: prompt.trim(),
    authHeader: 'Authorization', authPrefix: 'Bearer ', extraHeaders: {},
  };
}

export function providerInput(form, builtinPrompt) {
  return {
    id: form.id, name: form.name.trim(), kind: form.kind,
    baseUrl: form.baseUrl.trim(), model: form.model.trim(),
    apiKey: form.apiKey.trim() || null, enabled: form.enabled,
    autoApplyTags: form.autoApplyTags, autoMarkReviewed: form.autoMarkReviewed,
    minConfidence: form.minConfidence, maxTags: form.maxTags, language: form.language,
    systemPrompt: form.systemPrompt.trim() || builtinPrompt.trim(),
    authHeader: form.kind === 'openai_compatible' ? form.authHeader : '',
    authPrefix: form.kind === 'openai_compatible' ? form.authPrefix : '',
    extraHeaders: { ...form.extraHeaders },
  };
}
