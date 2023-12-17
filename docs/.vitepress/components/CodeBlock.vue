<template>
    <div class="language-" :style="styles">
		<Editor
			:modelValue="props.code"
			:options="{ ...(props.isCodeBlock ? CODEBLOCK_OPTIONS : EDITOR_OPTIONS), ...props.options }"
			:lang="lang"
			:isCodeBlock="props.isCodeBlock"
		/>
	</div>
</template>

<script setup lang="ts">
import type monacoEditor from 'monaco-editor/esm/vs/editor/editor.api';
import { ref, watch } from 'vue';

const props = withDefaults(defineProps<{ code: string, options?: monacoEditor.editor.IStandaloneEditorConstructionOptions, lang?: string, isCodeBlock?: boolean }>(), {
	isCodeBlock: true
})
defineEmits<{ (e: "update:modelValue", value: string): void }>()

const styles = ref()
watch(
	() => props.code,
	(code: string) => {
		styles.value = {
			width: "100%",
			height: Math.min(code.split("\n").length * (props.options?.lineHeight ?? 18), 460) + 40 + "px",
			padding: "20px 0px",
			background: props.isCodeBlock ? undefined : "transparent"
		};
	},
	{ immediate: true },
);
;

const EDITOR_OPTIONS: monacoEditor.editor.IStandaloneEditorConstructionOptions = { readOnly: true, scrollBeyondLastLine: false }
const CODEBLOCK_OPTIONS: monacoEditor.editor.IStandaloneEditorConstructionOptions = { ...EDITOR_OPTIONS, minimap: { enabled: false }, lineNumbers: "off", scrollbar: { vertical: "hidden", horizontal: "hidden", handleMouseWheel: false }, overviewRulerLanes: 0  }
</script>

