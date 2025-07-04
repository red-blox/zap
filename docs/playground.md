# Playground

<ClientOnly>

<div class="flex">
	<div class="button plugin-tabs">
		<button @click="copyURL"><span>📎</span> Copy URL</button>
	</div>
	<div class="button plugin-tabs">
		<button @click="toggleNoWarnings">{{ noWarnings ? "❌ Zap will Only Error" : "⚠️ Warnings Allowed" }}</button>
	</div>
</div>

**Input:**

<div class="editor plugin-tabs" :style="styles">
	<Editor v-model="code" />
</div>

**Output:**

<PluginTabs sharedStateKey="outputTab">
	<PluginTabsTab :label="!compiledResult.code ? 'Errors' : 'Warnings'" v-if="compiledResult.diagnostics">
		<pre>
			<Ansi class="ansi monaco-component" useClasses>{{ compiledResult.diagnostics }}</Ansi>
		</pre>
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
	<PluginTabsTab label="Tooling" v-if="compiledResult.code && compiledResult.code.tooling">
		<CodeBlock
			:code="compiledResult.code.tooling.code"
			lang="lua"
			:isCodeBlock="false"
		/>
	</PluginTabsTab>
	<PluginTabsTab label="Types" v-if="compiledResult.code && compiledResult.code.types">
		<CodeBlock
			:code="compiledResult.code.types.code"
			lang="lua"
			:isCodeBlock="false"
		/>
	</PluginTabsTab>
	<PluginTabsTab label="Types (TS)" v-if="isTypeScript && compiledResult.code && compiledResult.code.types">
		<CodeBlock
			:code="compiledResult.code.types.defs"
			lang="typescript"
			:isCodeBlock="false"
		/>
	</PluginTabsTab>
</PluginTabs>

</ClientOnly>

<script setup lang="ts">
import MonacoEditor from "@guolao/vue-monaco-editor";
import type { Monaco } from "@monaco-editor/loader";
import Ansi from "ansi-to-vue3";
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
const noWarnings = ref(false);
const isTypeScript = ref(false)
const free = () => {};
const compiledResult = ref<PlaygroundCode>({
	diagnostics: "Write some code to see output here!\n",
	free,
})

onMounted(() => {
	const codeParam = new URLSearchParams(window.location.search).get("code")

	if (codeParam) {
		sessionStorage.setItem("code", decodeURIComponent(codeParam))
		go("/playground")
		return;
	}

	const codeStr = sessionStorage.getItem("code") ?? ""

	try {
		const result = atob(codeStr)
		code.value = result
	} catch (err) {
		console.warn(err)
	}
})

const clamp = (number, min, max) => Math.max(min, Math.min(number, max));

watch([code, noWarnings], ([newCode, noWarnings]) => {
	try {
		compiledResult.value = run(newCode, noWarnings, true);

		if (compiledResult.value.code?.client.defs && compiledResult.value.code?.server.defs) {
			isTypeScript.value = true
		} else {
			isTypeScript.value = false
		}
	} catch (err) {
		compiledResult.value = {
			diagnostics: `Unable to compile code: ${err.message}`,
			free
		}

		isTypeScript.value = false
	}
	
	styles.value = {
		width: "100%",
		height: clamp(newCode.split("\n").length * 18, 260, 460) + 40 + "px",
		padding: "20px 0px",
	};

	sessionStorage.setItem("code", btoa(newCode))
})

const copyURL = () => {
	const result = encodeURIComponent(btoa(code.value))
	navigator.clipboard.writeText(`${location.protocol}//${location.host}/playground?code=${result}`)
}

const toggleNoWarnings = () => {
	noWarnings.value = !noWarnings.value
}
</script>

<style>
.editor {
	width: 100%;
	height: 60vh;
}
.flex {
	display: flex;
	gap: 16px;
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

.ansi {
	display: block;
	padding: 0px 16px;
	font-size: 12px;
	text-wrap: auto;
}
.ansi-bold {
	font-weight: bold
}
.ansi-yellow-fg {
	color: var(--vscode-charts-yellow)
}
.ansi-blue-fg {
	color: var(--vscode-charts-blue)
}
.ansi-red-fg {
	color: var(--vscode-charts-red)
}
.ansi-bright-yellow-fg {
	color: var(--vscode-editorWarning-foreground)
}
.ansi-bright-red-fg {
	color: var(--vscode-editorError-foreground)
}
</style>
