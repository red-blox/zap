<template>
	<MonacoEditor
		:value="props.modelValue"
		@update:value="(val: string) => $emit('update:modelValue', val)"
		:language="lang ?? 'zapConfig'"  
		:theme="`${props.isCodeBlock ? 'codeblock' : 'tab'}-${isDark ? 'dark' : 'light'}`"
		@beforeMount="beforeMount"
		:options="EDITOR_OPTIONS"
	/>	
</template>	

<script setup lang="ts">
import MonacoEditor from "@guolao/vue-monaco-editor";
import type { Monaco } from "@monaco-editor/loader"
import type monacoEditor from 'monaco-editor/esm/vs/editor/editor.api';
import { useData } from "vitepress";

const props = defineProps<{ modelValue: string, options?: monacoEditor.editor.IStandaloneEditorConstructionOptions, lang?: string, isCodeBlock?: boolean  }>()
defineEmits<{ (e: "update:modelValue", value: string): void }>()

const EDITOR_OPTIONS: monacoEditor.editor.IStandaloneEditorConstructionOptions = { ...props.options, formatOnPaste: true, formatOnType: true, stickyScroll: { enabled: true } }
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
		],
		surroundingPairs: [
			{ open: "{", close: "}" },
			{ open: "[", close: "]" },
			{ open: "(", close: ")" },
		],
	});

	const keywords = ["event", "opt", "type"] as const;

	const operators = ["true", "false"] as const;

	const Locations = ["Server", "Client"] as const;

	const Brand = ["Reliable", "Unreliable"] as const;

	const Calls = ["SingleSync", "SingleAsync", "ManySync", "ManyAsync"] as const;

	const Options = ["typescript", "writechecks", "casing"] as const;

	const Casing = ["PascalCase", "camelCase", "snake_case"] as const;

	const setting = [...Locations, ...Brand, ...Calls, ...Casing] as const;

	const types = [
		"u8",
		"u16",
		"u32",
		"u64",
		"i8",
		"i16",
		"i32",
		"i64",
		"f32",
		"f64",
		"bool",
		"string",
		"Instance",
		"Vector3"
	] as const;

	const eventParamToArray = {
		from: Locations,
		type: Brand,
		call: Calls,
		data: [],
	} as const;

	const wordToArray = {
		...eventParamToArray,

		opt: Options,

		casing: Casing,
		typescript: operators,
		writechecks: operators,
	} as const;

	monaco.languages.registerTokensProviderFactory("zapConfig", {
		create: () => ({
			defaultToken: "",

			keywords: [...keywords, ...operators],

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

					// keys
					[
						/({)(\s*)([a-zA-Z_]\w*)(\s*)(:)(?!:)/,
						["@brackets", "", "key", "", "delimiter"],
					],
					[
						/(,)(\s*)([a-zA-Z_]\w*)(\s*)(:)(?!:)/,
						["delimiter", "", "key", "", "delimiter"],
					],

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

			if (range.startColumn === 1) {
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
						label: "opt",
						kind: monaco.languages.CompletionItemKind.Snippet,
						insertText: "opt ${1} = ${2}\n",
						insertTextRules:
							monaco.languages.CompletionItemInsertTextRule
								.InsertAsSnippet,
						documentation: "Settings",
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
				];
				return { suggestions };
			} else {
				let i = -1;
				let wordBefore = model.getWordAtPosition({
					...position,
					column: word.startColumn + i,
				});
				// Go back until we get a word to determine what the autocomplete should be
				while (!wordBefore && word.startColumn + i > 0) {
					wordBefore = model.getWordAtPosition({
						...position,
						column: word.startColumn + i,
					});
					i--;
				}

				// for now, if there's no wordBefore we can assume it's the event object
				const arr = !wordBefore
					? Object.keys(eventParamToArray)
					: wordToArray[wordBefore.word] ?? types;

				const identifiers = arr.map((k) => ({
					label: k,
					insertText: k,
					kind: monaco.languages.CompletionItemKind.Variable,
					range,
				}));
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

