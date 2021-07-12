# Method chain

Quick and dirty detection of method chains in Java projects. 

Takes a directory with Java projects. Opens each directory, and for each `.java` file found inside, processes the file and calculates method chain lengths. Produces a CSv file with a histogram of chain lengths for each projects.

## Build

```bash
cargo build --release
```

## Run

```
cargo run --release -- --project-dir PROJECT_DIR_PATH --output-path OUTPUT_PATH
```

Where `PROJECT_DIR_PATH` is a directory containing Java projects and `OUTPUT_PATH` is a CSV file where the histograms are written to.

## Output format

The output CSV file has three columns: `project`, `chain length`, and `frequency`. The `project` column reflects the name of the directory in the project directory.

For an example project directory with this structure (`PushpinderSinghGrewal/lan-chat-app`, `shashirajraja/onlinebookstore`
, `HouariZegai/Calculator`):

```
projects
├── Calculator
│   ├── LICENSE
│   ├── pom.xml
│   ├── README.md
│   ├── screenshots
│   └── src
├── lan-chat-app
│   ├── build.xml
│   ├── lanchatapp.jpeg
│   ├── manifest.mf
│   ├── nbproject
│   ├── README.md
│   └── src
└── onlinebookstore
    ├── Dummy_Database.md
    ├── OnlineBookStore
    ├── pom.xml
    ├── README.md
    ├── setup
    └── WebContent
```

The toolk produces the following output:

```
project, chain length, frequency
lan-chat-app, 7, 1
lan-chat-app, 6, 2
lan-chat-app, 4, 5
lan-chat-app, 3, 11
lan-chat-app, 2, 10
lan-chat-app, 1, 205
onlinebookstore, 1, 226
Calculator, 2, 8
Calculator, 1, 416
```
