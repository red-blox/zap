# Playground

<br>

**Input:**

<div class="editor plugin-tabs" :style="styles">
	<Editor v-model="code" />
</div>

**Output:**

:::tabs
== Client
<CodeBlock
	:code="compiledResult.client"
	language="lua"
	scroll
	:isCodeBlock="false"
/>
== Server
<CodeBlock
	:code="compiledResult.server"
	language="lua"
	scroll
	:isCodeBlock="false"
/>
:::

<script setup lang="ts">
import MonacoEditor from "@guolao/vue-monaco-editor";
import type { Monaco } from "@monaco-editor/loader";
import { useData, useRoute } from "vitepress";
import { ref, watch } from "vue";
import { run_wasm, Code } from "../zap/pkg"

const { isDark } = useData();
const { path, data } = useRoute();

const styles = ref({
	width: "100%",
	height: "300px",
	padding: "20px 0px",
})
const code = ref("");
const compiledResult = ref<Code>({
	client: "-- Write some code to see output here!\n",
	server: "-- Write some code to see output here!\n",
	free: () => {}
})

const clamp = (number, min, max) => Math.max(min, Math.min(number, max));

watch(code, (newCode) => {
	try {
		compiledResult.value = run_wasm(newCode);

		if (!compiledResult.value.client && !compiledResult.value.server) compiledResult.value = {
			client: "-- Add an event to see output here!\n",
			server: "-- Add an event to see output here!\n",
			free: () => {}
		}
	} catch (err) {
		compiledResult.value = {
			client: `--[[\n${err.message}\n]]`,
			server: `--[[\n${err.message}\n]]`,
			free: () => {}
		}
	}
	
	styles.value = {
		width: "100%",
		height: clamp(newCode.split("\n").length * 18, 260, 460) + 40 + "px",
		padding: "20px 0px",
	};
})

</script>

<style>
.editor {
	width: 100%;
	height: 60vh;
}
</style>
