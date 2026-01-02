const {
  override,
  addBundleVisualizer,
} = require("customize-cra");
const { DefinePlugin } = require('webpack');

const SentryPlugin = require("@sentry/webpack-plugin");
const CompressionPlugin = require("compression-webpack-plugin");
const zlib = require("zlib");

const release = Math.floor(Date.now() / 1000).toString()

const addSentryPlugin = config => {
    config.plugins.push(
        new DefinePlugin({
          RELEASE: JSON.stringify(release),
          SENTRY_DSN: JSON.stringify(process.env.SENTRY_DSN)
        }),
        new CompressionPlugin({
            filename: "[path][base].gz",
            algorithm: "gzip",
            test: /\.(js|css|html|svg)$/,
            threshold: 0,
            minRatio: 0.8,
        }),
        new CompressionPlugin({
            filename: "[path][base].br",
            algorithm: "brotliCompress",
            test: /\.(js|css|html|svg)$/,
            compressionOptions: {
                params: {
                    [zlib.constants.BROTLI_PARAM_QUALITY]: zlib.constants.BROTLI_MAX_QUALITY,
                },
            },
            threshold: 0,
            minRatio: 0.8,
        })
    );
    return config;
}


module.exports = override(
    // add webpack bundle visualizer if BUNDLE_VISUALIZE flag is enabled
    /*process.env.BUNDLE_VISUALIZE == 1 && */
    // addBundleVisualizer(),
    addSentryPlugin,

);
