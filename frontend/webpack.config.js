const path = require('path');
const HtmlWebpackPlugin = require('html-webpack-plugin');
const MiniCssExtractPlugin = require('mini-css-extract-plugin');
const CssMinimizerPlugin = require('css-minimizer-webpack-plugin');
const TerserPlugin = require('terser-webpack-plugin');
const { CleanWebpackPlugin } = require('clean-webpack-plugin');
const CopyWebpackPlugin = require('copy-webpack-plugin');
const CompressionPlugin = require('compression-webpack-plugin');

const isProduction = process.env.NODE_ENV === 'production';

module.exports = {
  mode: isProduction ? 'production' : 'development',
  
  entry: {
    // Core application files
    core: './assets/js/core/index.js',
    game: './assets/js/game/index.js',
    components: './assets/js/components/index.js',
    
    // Individual pages for code splitting
    desktop: './assets/js/pages/desktop.js',
    processes: './assets/js/pages/processes.js',
    software: './assets/js/pages/software.js',
    hardware: './assets/js/pages/hardware.js',
    internet: './assets/js/pages/internet.js',
    banking: './assets/js/pages/banking.js',
    clan: './assets/js/pages/clan.js',
    
    // Vendor libraries
    vendor: ['lodash', 'moment', 'chart.js']
  },
  
  output: {
    path: path.resolve(__dirname, 'dist'),
    filename: isProduction ? 'js/[name].[contenthash:8].js' : 'js/[name].js',
    chunkFilename: isProduction ? 'js/[name].[contenthash:8].chunk.js' : 'js/[name].chunk.js',
    publicPath: '/',
    clean: true
  },
  
  resolve: {
    alias: {
      '@': path.resolve(__dirname, 'assets/js'),
      '@css': path.resolve(__dirname, 'assets/css'),
      '@images': path.resolve(__dirname, 'assets/images'),
      '@sounds': path.resolve(__dirname, 'assets/sounds'),
      '@templates': path.resolve(__dirname, 'templates')
    },
    extensions: ['.js', '.json', '.css']
  },
  
  module: {
    rules: [
      // JavaScript
      {
        test: /\.js$/,
        exclude: /node_modules/,
        use: {
          loader: 'babel-loader',
          options: {
            presets: [
              ['@babel/preset-env', {
                targets: {
                  browsers: ['> 1%', 'last 2 versions', 'not dead']
                },
                useBuiltIns: 'usage',
                corejs: 3
              }]
            ],
            plugins: [
              '@babel/plugin-proposal-class-properties',
              '@babel/plugin-proposal-optional-chaining',
              '@babel/plugin-proposal-nullish-coalescing-operator'
            ]
          }
        }
      },
      
      // CSS
      {
        test: /\.css$/,
        use: [
          isProduction ? MiniCssExtractPlugin.loader : 'style-loader',
          'css-loader',
          {
            loader: 'postcss-loader',
            options: {
              postcssOptions: {
                plugins: [
                  ['autoprefixer'],
                  ...(isProduction ? [['cssnano']] : [])
                ]
              }
            }
          }
        ]
      },
      
      // SCSS
      {
        test: /\.scss$/,
        use: [
          isProduction ? MiniCssExtractPlugin.loader : 'style-loader',
          'css-loader',
          {
            loader: 'postcss-loader',
            options: {
              postcssOptions: {
                plugins: [
                  ['autoprefixer'],
                  ...(isProduction ? [['cssnano']] : [])
                ]
              }
            }
          },
          'sass-loader'
        ]
      },
      
      // Images
      {
        test: /\.(png|jpe?g|gif|svg|webp|avif)$/i,
        type: 'asset',
        parser: {
          dataUrlCondition: {
            maxSize: 8192 // 8kb
          }
        },
        generator: {
          filename: 'images/[name].[contenthash:8][ext]'
        },
        use: isProduction ? [
          {
            loader: 'image-webpack-loader',
            options: {
              mozjpeg: {
                progressive: true,
                quality: 80
              },
              optipng: {
                optimizationLevel: 7
              },
              pngquant: {
                quality: [0.65, 0.8],
                speed: 4
              },
              gifsicle: {
                optimizationLevel: 3
              },
              webp: {
                quality: 80
              }
            }
          }
        ] : []
      },
      
      // Fonts
      {
        test: /\.(woff2?|eot|ttf|otf)$/i,
        type: 'asset/resource',
        generator: {
          filename: 'fonts/[name].[contenthash:8][ext]'
        }
      },
      
      // Audio
      {
        test: /\.(mp3|wav|ogg|m4a)$/i,
        type: 'asset/resource',
        generator: {
          filename: 'sounds/[name].[contenthash:8][ext]'
        }
      },
      
      // HTML Templates
      {
        test: /\.html$/,
        use: ['html-loader']
      }
    ]
  },
  
  plugins: [
    // Clean dist folder
    new CleanWebpackPlugin(),
    
    // Extract CSS
    ...(isProduction ? [
      new MiniCssExtractPlugin({
        filename: 'css/[name].[contenthash:8].css',
        chunkFilename: 'css/[name].[contenthash:8].chunk.css'
      })
    ] : []),
    
    // Generate HTML files
    new HtmlWebpackPlugin({
      template: './templates/layout.html',
      filename: 'index.html',
      chunks: ['core', 'vendor'],
      minify: isProduction ? {
        removeComments: true,
        collapseWhitespace: true,
        removeRedundantAttributes: true,
        useShortDoctype: true,
        removeEmptyAttributes: true,
        removeStyleLinkTypeAttributes: true,
        keepClosingSlash: true,
        minifyJS: true,
        minifyCSS: true,
        minifyURLs: true
      } : false
    }),
    
    new HtmlWebpackPlugin({
      template: './templates/pages/public/login.html',
      filename: 'login.html',
      chunks: ['core', 'vendor'],
      minify: isProduction
    }),
    
    new HtmlWebpackPlugin({
      template: './templates/pages/public/register.html',
      filename: 'register.html',
      chunks: ['core', 'vendor'],
      minify: isProduction
    }),
    
    new HtmlWebpackPlugin({
      template: './templates/pages/public/home.html',
      filename: 'home.html',
      chunks: ['core', 'vendor'],
      minify: isProduction
    }),
    
    // Copy static assets
    new CopyWebpackPlugin({
      patterns: [
        {
          from: 'assets/images',
          to: 'images',
          noErrorOnMissing: true
        },
        {
          from: 'assets/sounds',
          to: 'sounds',
          noErrorOnMissing: true
        },
        {
          from: 'assets/fonts',
          to: 'fonts',
          noErrorOnMissing: true
        },
        {
          from: 'public',
          to: '',
          noErrorOnMissing: true
        }
      ]
    }),
    
    // Gzip compression for production
    ...(isProduction ? [
      new CompressionPlugin({
        algorithm: 'gzip',
        test: /\.(js|css|html|svg)$/,
        threshold: 8192,
        minRatio: 0.8
      })
    ] : [])
  ],
  
  optimization: {
    minimize: isProduction,
    minimizer: [
      new TerserPlugin({
        terserOptions: {
          parse: {
            ecma: 8
          },
          compress: {
            ecma: 5,
            warnings: false,
            comparisons: false,
            inline: 2,
            drop_console: true,
            drop_debugger: true
          },
          mangle: {
            safari10: true
          },
          output: {
            ecma: 5,
            comments: false,
            ascii_only: true
          }
        }
      }),
      new CssMinimizerPlugin()
    ],
    
    // Code splitting
    splitChunks: {
      chunks: 'all',
      cacheGroups: {
        // Vendor libraries
        vendor: {
          test: /[\\/]node_modules[\\/]/,
          name: 'vendor',
          chunks: 'all',
          priority: 10
        },
        
        // Common code
        common: {
          name: 'common',
          minChunks: 2,
          chunks: 'all',
          priority: 5,
          reuseExistingChunk: true
        },
        
        // CSS
        styles: {
          name: 'styles',
          test: /\.css$/,
          chunks: 'all',
          enforce: true
        }
      }
    },
    
    // Runtime chunk
    runtimeChunk: {
      name: 'runtime'
    }
  },
  
  devtool: isProduction ? 'source-map' : 'eval-source-map',
  
  devServer: {
    static: {
      directory: path.join(__dirname, 'dist'),
      publicPath: '/'
    },
    port: 3000,
    host: '0.0.0.0',
    hot: true,
    open: true,
    compress: true,
    historyApiFallback: {
      rewrites: [
        { from: /^\/login/, to: '/login.html' },
        { from: /^\/register/, to: '/register.html' },
        { from: /./, to: '/index.html' }
      ]
    },
    client: {
      overlay: {
        warnings: false,
        errors: true
      },
      progress: true
    },
    proxy: {
      '/api': {
        target: 'http://localhost:8080',
        changeOrigin: true,
        secure: false
      },
      '/ws': {
        target: 'ws://localhost:8080',
        ws: true,
        changeOrigin: true
      }
    }
  },
  
  performance: {
    hints: isProduction ? 'warning' : false,
    maxEntrypointSize: 250000,
    maxAssetSize: 250000
  },
  
  stats: {
    colors: true,
    modules: false,
    children: false,
    chunks: false,
    chunkModules: false
  }
};