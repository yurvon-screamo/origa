/** @type {import('tailwindcss').Config} */
module.exports = {
	content: ["./src/**/*.rs", "./index.html"],
	theme: {
		extend: {
			padding: {
				"safe-t": "env(safe-area-inset-top, 0px)",
				"safe-b": "env(safe-area-inset-bottom, 0px)",
				"safe-l": "env(safe-area-inset-left, 0px)",
				"safe-r": "env(safe-area-inset-right, 0px)",
				"safe-x":
					"env(safe-area-inset-left, 0px) env(safe-area-inset-right, 0px)",
				"safe-y":
					"env(safe-area-inset-top, 0px) env(safe-area-inset-bottom, 0px)",
				safe: "env(safe-area-inset-top, 0px) env(safe-area-inset-right, 0px) env(safe-area-inset-bottom, 0px) env(safe-area-inset-left, 0px)",
			},
			margin: {
				"safe-t": "env(safe-area-inset-top, 0px)",
				"safe-b": "env(safe-area-inset-bottom, 0px)",
			},
		},
	},
	plugins: [],
};
