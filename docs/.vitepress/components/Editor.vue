<template>
	<MonacoEditor
		:value="props.modelValue"
		@update:value="(val: string) => $emit('update:modelValue', val)"
		:language="lang ?? 'zapConfig'"  
		:theme="`${props.isCodeBlock ? 'codeblock' : 'tab'}-${isDark ? 'dark' : 'light'}`"
		@beforeMount="beforeMount"
		@mount="(editor) => $emit('mounted', editor)"
		:options="EDITOR_OPTIONS"
	/>	
</template>	

<script setup lang="ts">
import MonacoEditor from "@guolao/vue-monaco-editor";
import type { Monaco } from "@monaco-editor/loader"
import type monacoEditor from 'monaco-editor/esm/vs/editor/editor.api';
import { useData } from "vitepress";
import type { Ref } from "vue";

const props = defineProps<{ modelValue: string, options?: monacoEditor.editor.IStandaloneEditorConstructionOptions, lang?: string, isCodeBlock?: boolean, lineHeight?: Ref<number>  }>()
defineEmits<{ (e: "update:modelValue", value: string): void, (e: "mounted", editor: monacoEditor.editor.IStandaloneCodeEditor): void }>()

const EDITOR_OPTIONS: monacoEditor.editor.IStandaloneEditorConstructionOptions = { ...props.options, formatOnPaste: true, formatOnType: true, stickyScroll: { enabled: true }, minimap: { enabled: false }, tabSize: 4 }
const { isDark } = useData();

const beforeMount = (monaco: Monaco) => {
	monaco.editor.defineTheme("tab-light", {
        base: "vs",
        inherit: true,
        colors: {
            "editor.background": "#f6f6f7",
			"editor.lineHighlightBorder": "#f6f6f7",
        },
        rules: [],
    })

	monaco.editor.defineTheme("tab-dark", {
        base: "vs-dark",
        inherit: true,
        colors: {
            "editor.background": "#202127",
			"editor.lineHighlightBorder": "#202127",
        },
        rules: [],
    })

	monaco.editor.defineTheme("codeblock-light", {
        base: "vs",
        inherit: true,
        colors: {
            "editor.background": "#f6f6f7",
			"editor.lineHighlightBorder": "#f6f6f7",
        },
        rules: [],
    })

	monaco.editor.defineTheme("codeblock-dark", {
        base: "vs-dark",
        inherit: true,
        colors: {
            "editor.background": "#161618",
			"editor.lineHighlightBorder": "#161618",
        },
        rules: [],
    })

	if (props.lang && props.lang !== "zapConfig") return;
	// is zapConfig already registered?
	if (monaco.languages.getLanguages().some(({ id }) => id === "zapConfig")) return;

	// Register a new language
	monaco.languages.register({ id: "zapConfig" });

	// Register a tokens provider for the language
	monaco.languages.setLanguageConfiguration("zapConfig", {
		comments: {
			lineComment: "--",
			blockComment: ["--[[", "]]"],
		},
		brackets: [
			["{", "}"],
			["[", "]"],
			["(", ")"],
		],
		autoClosingPairs: [
			{ open: "{", close: "}" },
			{ open: "[", close: "]" },
			{ open: "(", close: ")" },
			{ open: '"', close: '"' },
			{ open: "'", close: "'" },
		],
		surroundingPairs: [
			{ open: "{", close: "}" },
			{ open: "[", close: "]" },
			{ open: "(", close: ")" },
			{ open: '"', close: '"' },
			{ open: "'", close: "'" },
		],
	});

	const Keywords = ["event", "opt", "type", "funct", "namespace"] as const;

	const TypeKeywords = ["enum", "struct", "map", "set"] as const;

	const Operators = ["true", "false"] as const;

	const Locations = ["Server", "Client"] as const;

	const Brand = ["Reliable", "Unreliable", "OrderedUnreliable"] as const;

	const Calls = ["SingleSync", "SingleAsync", "ManySync", "ManyAsync", "Polling"] as const;

	const Options = ["write_checks", "typescript", "typescript_max_tuple_length", "typescript_enum", "manual_event_loop", "remote_scope", "remote_folder", "server_output", "client_output", "casing", "yield_type", "async_lib", "tooling", "tooling_output", "tooling_show_internal_data", "disable_fire_all", "types_output", "call_default"] as const;

	const TypeScriptEnum = ["StringLiteral", "ConstEnum", "StringConstEnum"].map((value) => `"${value}"`)
	const Casing = ["PascalCase", "camelCase", "snake_case"].map((value) => `"${value}"`);
	const YieldType = ["yield", "future", "promise"].map((value) => `"${value}"`);

	const setting = [...Locations, ...Brand, ...Calls, ...Casing] as const;

	const types = [
		"u8",
		"u16",
		"u32",
		"i8",
		"i16",
		"i32",
		"f32",
		"f64",
		"boolean",
		"string",
		"buffer",
		"unknown",
		"Instance",
		"Color3",
		"Vector3",
		"vector",
		"AlignedCFrame",
		"CFrame",
	] as const;

	const EventParamToArray = {
		from: Locations,
		type: null,
		call: Calls,
		data: null,
	} as const;

	const FunctionParamToArray = {
		call: Calls,
		args: null,
		rets: null,
	} as const

	const WordToArray = {
		...EventParamToArray,
		...FunctionParamToArray,

		opt: Options,

		typescript: Operators,
		typescript_max_tuple_length: [],
		typescript_enum: TypeScriptEnum,

		tooling: Operators,
		tooling_show_internal_data: Operators,

		write_checks: Operators,
		manual_event_loop: Operators,

		remote_scope: [],
		remote_folder: [],

		server_output: [],
		client_output: [],
		types_output: [],
		tooling_output: [],

		casing: Casing,
		yield_type: YieldType,
		async_lib: [],
		disable_fire_all: Operators,

		call_default: Calls.map((value) => `"${value}"`)
	} as const;

	const KeywordToArray = {
		event: Object.keys(EventParamToArray),
		funct: Object.keys(FunctionParamToArray),
		namespace: [],
		type: []
	} as const

	monaco.languages.registerTokensProviderFactory("zapConfig", {
		create: () => ({
			defaultToken: "",

			keywords: [...Keywords, ...TypeKeywords, ...Operators],

			brackets: [
				{ token: "delimiter.bracket", open: "{", close: "}" },
				{ token: "delimiter.array", open: "[", close: "]" },
				{ token: "delimiter.parenthesis", open: "(", close: ")" },
			],

			operators: ["=", ":", ",", ".."],

			types,

			setting,

			symbols: /[=:,]|\.\.+/,

			// The main tokenizer for our languages
			tokenizer: {
				root: [
					// numbers
					[/\d+?/, "number"],

					// delimiters and operators
					[/[{}()\[\]]/, "@brackets"],
					[
						/@symbols/,
						{
							cases: {
								"@operators": "delimiter",
								"@default": "",
							},
						},
					],

					// "str" identifiers
					[/\"\w+\"/, "regexp"],

					// identifiers and keywords
					[/(\w+):/, "identifier"],
					[
						/[a-zA-Z_]\w*/,
						{
							cases: {
								"@keywords": { token: "keyword.$0" },
								"@setting": "type.identifier",
								"@types": "string",
								"@default": "variable",
							},
						},
					],

					// whitespace
					{ include: "@whitespace" },
				],

				whitespace: [
					[/[ \t\r\n]+/, ""],
					[/--\[([=]*)\[/, "comment", "@comment.$1"],
					[/--.*$/, "comment"],
				],

				comment: [
					[/[^\]]+/, "comment"],
					[
						/\]([=]*)\]/,
						{
							cases: {
								"$1==$S2": { token: "comment", next: "@pop" },
								"@default": "comment",
							},
						},
					],
					[/./, "comment"],
				],
			},
		}),
	});

	// Register a completion item provider for the new language
	monaco.languages.registerCompletionItemProvider("zapConfig", {
		provideCompletionItems: (model, position) => {
			var word = model.getWordUntilPosition(position);
			var range = {
				startLineNumber: position.lineNumber,
				endLineNumber: position.lineNumber,
				startColumn: word.startColumn,
				endColumn: word.endColumn,
			};

			let i = -1;
			let wordBefore = model.getWordAtPosition({
				...position,
				column: word.startColumn + i,
			});

			// Go back in the line until we get a word to determine what the autocomplete should be
			while (!wordBefore && word.startColumn + i > 0) {
				wordBefore = model.getWordAtPosition({
					...position,
					column: word.startColumn + i,
				});
				i--;
			}

			const column = word.startColumn - (word.startColumn % 4 ? 4 : 1);
			let lineNumber = position.lineNumber - 1;

			// if unsucessful, look for the opening statement of the block (event, funct, namespace)
			while (!wordBefore && lineNumber > 0) {
				wordBefore = model.getWordAtPosition({
					column,
					lineNumber,
				});
				lineNumber--;
			}

			if (range.startColumn === 1 || (wordBefore?.word === "namespace" && lineNumber !== position.lineNumber - 1)) {
				var suggestions = [
					{
						label: "type",
						kind: monaco.languages.CompletionItemKind.Snippet,
						insertText: "type ${1} = ${2}\n",
						insertTextRules:
							monaco.languages.CompletionItemInsertTextRule
								.InsertAsSnippet,
						documentation: "Type Statement",
						range: range,
					},
					{
						label: "event",
						kind: monaco.languages.CompletionItemKind.Snippet,
						insertText: [
							"event ${1} = {",
							"\tfrom: ${2},",
							"\ttype: ${3},",
							"\tcall: ${4},",
							"\tdata: ${5}",
							"}\n",
						].join("\n"),
						insertTextRules:
							monaco.languages.CompletionItemInsertTextRule
								.InsertAsSnippet,
						documentation: "Event",
						range: range,
					},
					{
						label: "funct",
						kind: monaco.languages.CompletionItemKind.Snippet,
						insertText: [
							"funct ${1} = {",
							"\tcall: ${2},",
							"\targs: ${3},",
							"\trets: ${4},",
							"}\n",
						].join("\n"),
						insertTextRules:
							monaco.languages.CompletionItemInsertTextRule
								.InsertAsSnippet,
						documentation: "Event",
						range: range,
					},
					{
						label: "namespace",
						kind: monaco.languages.CompletionItemKind.Snippet,
						insertText: [
							"namespace ${1} = {",
							"\t${2}",
							"}\n",
						].join("\n"),
						insertTextRules:
							monaco.languages.CompletionItemInsertTextRule
								.InsertAsSnippet,
						documentation: "Namespace",
						range: range,
					},
				];

				if (range.startColumn == 1) {
					suggestions.push({
						label: "opt",
						kind: monaco.languages.CompletionItemKind.Snippet,
						insertText: "opt ${1} = ${2}\n",
						insertTextRules:
							monaco.languages.CompletionItemInsertTextRule
								.InsertAsSnippet,
						documentation: "Settings",
						range: range,
					})
				}

				return { suggestions };
			} else if (wordBefore) {
				const wordToArray = WordToArray[wordBefore.word];
				const keywordToArray = KeywordToArray[wordBefore.word]

				let arr;

				if (wordToArray) {
					arr = wordToArray
				} else if (keywordToArray) {
					// special case for "type" as it can be Brand or types
					if (wordBefore.word === "type" && model.getLineContent(lineNumber + 1).includes(":")) {
						arr = Brand;
					} else if (lineNumber === position.lineNumber - 1) {
						arr = []
					} else {
						arr = keywordToArray
					}
				} else {
					arr = types
				}

				const identifiers = arr.map((k) => ({
					label: k,
					insertText: k,
					kind: monaco.languages.CompletionItemKind.Variable,
					range,
				}));

				if (!wordToArray && !keywordToArray && arr) {
					identifiers.push(
						{
							label: "enum",
							kind: monaco.languages.CompletionItemKind.Variable,
							insertText: "enum ",
							range: range,
						},
						{
							label: "map",
							kind: monaco.languages.CompletionItemKind.Snippet,
							insertText: "map { [${1}]: ${2} }\n",
							insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
							documentation: "Map",
							range: range,
						},
						{
							label: "set",
							kind: monaco.languages.CompletionItemKind.Snippet,
							insertText: "set { $1 }\n",
							insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
							documentation: "Set",
							range: range,
						},
						{
							label: "struct",
							kind: monaco.languages.CompletionItemKind.Snippet,
							insertText: ["struct {", "\t${1}: ${2},", "}"].join("\n"),
							insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
							documentation: "Struct",
							range: range,
						}
					);
				}

				return { suggestions: identifiers };
			}
		},
	});
};
</script>

<style>
.editor {
	height: 100%;
	width: 100%;
}
</style>
