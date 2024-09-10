const tailwindcss = require('tailwindcss');
const autoprefixer = require('autoprefixer');

// TODO: Figure out a way to nest all the styles

const config = {
	plugins: [
		//Some plugins, like tailwindcss/nesting, need to run before Tailwind,
		tailwindcss(),
		//But others, like autoprefixer, need to run after,
		autoprefixer
	]
};

module.exports = config;