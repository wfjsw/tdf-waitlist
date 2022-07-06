const {
  override,
  addBundleVisualizer,
} = require("customize-cra");
const { DefinePlugin } = require('webpack');

const SentryPlugin = require("@sentry/webpack-plugin");

const release = Math.floor(Date.now() / 1000).toString()

const addSentryPlugin = config => {
    config.devtool = 'hidden-source-map';
    config.plugins.push(
        new SentryPlugin({
            release: process.env.RELEASE,
            include: "./build",
            url: 'https://bugreport.indexyz.me/',
            org: 'sentry',
            project: 'waitlist-frontend',
            release,
        }),
        new DefinePlugin({
            RELEASE: JSON.stringify(release),
        }),
    );
    return config;
}


module.exports = override(
    // add webpack bundle visualizer if BUNDLE_VISUALIZE flag is enabled
    /*process.env.BUNDLE_VISUALIZE == 1 && */
    addBundleVisualizer(),
    addSentryPlugin,

);
