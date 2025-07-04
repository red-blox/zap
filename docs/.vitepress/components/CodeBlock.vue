<template>
    <div class="language-" :style="styles">
		<button v-if="props.isCodeBlock && (lang === 'zapConfig' || lang === undefined)" class="tooltip" @click="showInPlayground">
			Display in Playground
		</button>
		<Editor
			:modelValue="props.code"
			:options="{ ...(props.isCodeBlock ? CODEBLOCK_OPTIONS : EDITOR_OPTIONS), ...props.options }"
			:lang="lang"
			:isCodeBlock="props.isCodeBlock"
			@mounted="onMount"
		/>
	</div>
</template>

<script setup lang="ts">
import type monacoEditor from 'monaco-editor/esm/vs/editor/editor.api';
import { ref, watch } from 'vue';
import { useRouter } from 'vitepress'

const props = withDefaults(defineProps<{ code: string, options?: monacoEditor.editor.IStandaloneEditorConstructionOptions, lang?: string, isCodeBlock?: boolean }>(), {
	isCodeBlock: true
})
defineEmits<{ (e: "update:modelValue", value: string): void }>()

const { go } = useRouter()

const styles = ref()
const lineHeight = ref(props.options?.lineHeight ?? 18)

const onMount = (editor: monacoEditor.editor.IStandaloneCodeEditor) => {
	lineHeight.value = editor.getOption(65)
}

watch(
	[() => props.code, lineHeight],
	([code, lineHeight]) => {
		styles.value = {
			width: "100%",
			height: Math.min(code.split("\n").length * lineHeight, 460) + 40 + "px",
			padding: "20px 0px",
			background: props.isCodeBlock ? undefined : "transparent",
			position: "relative",
		};
	},
	{ immediate: true },
);
;

const showInPlayground = () => {
	try {
		const link = encodeURIComponent(btoa(props.code))
		go(`/playground?code=${link}`)
	} catch (err) {

	}
}

const EDITOR_OPTIONS: monacoEditor.editor.IStandaloneEditorConstructionOptions = { readOnly: true, scrollBeyondLastLine: false }
const CODEBLOCK_OPTIONS: monacoEditor.editor.IStandaloneEditorConstructionOptions = { ...EDITOR_OPTIONS, minimap: { enabled: false }, lineNumbers: "off", scrollbar: { vertical: "hidden", horizontal: "hidden", handleMouseWheel: false }, overviewRulerLanes: 0  }
</script>

<style>
.tooltip {
	position: absolute;
	z-index: 10;
	background: var(--vp-c-default-soft);
	border-radius: 8px;
	padding: 4px 8px;
	right: 12px;
	top: 12px;
	opacity: 0;
	transition: opacity 0.3s;
}

.language-:hover > .tooltip {
	opacity: 1
}
</style>
