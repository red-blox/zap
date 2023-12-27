# Playground

<ClientOnly>

<div class="button plugin-tabs">
	<button @click="saveURL"><span>📎</span> Save URL</button>
</div>

**Input:**

<div class="editor plugin-tabs" :style="styles">
	<Editor v-model="code" />
</div>

**Output:**

<PluginTabs sharedStateKey="outputTab">
	<PluginTabsTab :label="!compiledResult.code ? 'Errors' : 'Warnings'" v-if="compiledResult.diagnostics">
		<CodeBlock
			:code="compiledResult.diagnostics"
			lang="text"
			:isCodeBlock="false"
		/>
	</PluginTabsTab>
	<PluginTabsTab label="Client" v-if="compiledResult.code">
		<CodeBlock
			:code="compiledResult.code.client.code"
			lang="lua"
			:isCodeBlock="false"
		/>
	</PluginTabsTab>
	<PluginTabsTab label="Client (TS)" v-if="isTypeScript && compiledResult.code">
		<CodeBlock
			:code="compiledResult.code.client.defs"
			lang="typescript"
			:isCodeBlock="false"
		/>
	</PluginTabsTab>
	<PluginTabsTab label="Server" v-if="compiledResult.code">
		<CodeBlock
			:code="compiledResult.code.server.code"
			lang="lua"
			:isCodeBlock="false"
		/>
	</PluginTabsTab>
	<PluginTabsTab label="Server (TS)" v-if="isTypeScript && compiledResult.code">
		<CodeBlock
			:code="compiledResult.code.server.defs"
			lang="typescript"
			:isCodeBlock="false"
		/>
	</PluginTabsTab>
</PluginTabs>

</ClientOnly>

<script setup lang="ts">
import MonacoEditor from "@guolao/vue-monaco-editor";
import type { Monaco } from "@monaco-editor/loader";
import { useData, useRouter } from "vitepress";
import { ref, watch, onMounted } from "vue";
import { run } from "../zap/package";
import type { Return as PlaygroundCode } from "../zap/package";

const { isDark } = useData();
const { go } = useRouter();

const styles = ref({
	width: "100%",
	height: "300px",
	padding: "20px 0px",
})
const code = ref("");
const isTypeScript = ref(false)
const free = () => {};
const compiledResult = ref<PlaygroundCode>({
	diagnostics: "Write some code to see output here!\n",
	free,
})

onMounted(() => {
	const codeParam = new URLSearchParams(window.location.search).get("code")
	const storedCode = localStorage.getItem("code")

	let codeStr = ""

	if (!codeParam && !!storedCode) {
		codeStr = storedCode;
		go(`/playground?code=${storedCode}`);
	} else if (codeParam) {
		codeStr = codeParam;
	}

	try {
		const result = atob(codeStr)
		code.value = result
	} catch (err) {
		console.warn(err)
	}
})

const clamp = (number, min, max) => Math.max(min, Math.min(number, max));

watch(code, (newCode) => {
	try {
		compiledResult.value = run(newCode);

		if (compiledResult.value.code?.client.defs && compiledResult.value.code?.server.defs) {
			isTypeScript.value = true
		} else {
			isTypeScript.value = false
		}
	} catch (err) {
		compiledResult.value = {
			diagnostics: `--[[\n${err.message}\n]]`,
			free
		}

		isTypeScript.value = false
	}
	
	styles.value = {
		width: "100%",
		height: clamp(newCode.split("\n").length * 18, 260, 460) + 40 + "px",
		padding: "20px 0px",
	};

	localStorage.setItem("code", btoa(newCode))
})

const saveURL = () => {
	const result = btoa(code.value)

	localStorage.setItem("code", result)
	navigator.clipboard.writeText(`${location.protocol}//${location.host}/playground?code=${result}`)

	go(`/playground?code=${result}`)
}
</script>

<style>
.editor {
	width: 100%;
	height: 60vh;
}

.button {
	padding: 12px;
	width: fit-content;
	transition: 0.2s transform
}
.button button {
	font-weight: 700
}
.button span {
	margin-right: 8px
}

.button:hover {
	transform: scale(1.1)
}
</style>
