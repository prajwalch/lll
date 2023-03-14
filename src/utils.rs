pub const PAGE_TEMPLATE: &str = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta http-equiv="X-UA-Compatible" content="IE=edge">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title}</title>
    <style>
        body {
            background-color: #000000;
            color: #f5f5f5;
            padding: 0px 50px;
        }

        a {
            display: block;
            color: #948bff;
            padding: 2px 10px;
            border-radius: 5px;
            text-decoration: none;
        }
        
        a:hover {
            background-color: #090909;
        }

        ul {
            padding: 10px 0px;
            border-top: 1px solid #1a1a1a;
            border-bottom: 1px solid #1a1a1a;
        }
        
        li {
            list-style-type: none;
        }
        
        li:nth-child(n + 2) {            
            margin-top: 5px;
        }

        svg {
            vertical-align: middle;
        }
    </style>
</head>
<body>
    {content}
</body>
</html>
"#;

pub const FILE_SVG_ICON: &str = r##"
<svg width="40px" height="40px" viewBox="0 0 1024 1024" class="icon" version="1.1"
    xmlns="http://www.w3.org/2000/svg">
    <path
        d="M576 102.4H268.8c-14.08 0-25.6 11.52-25.6 25.6v742.4c0 14.08 11.52 25.6 25.6 25.6h512c14.08 0 25.6-11.52 25.6-25.6V332.8L576 102.4z"
        fill="#00B2AE" />
    <path
        d="M780.8 908.8H268.8c-21.76 0-38.4-16.64-38.4-38.4V128c0-21.76 16.64-38.4 38.4-38.4h312.32L819.2 327.68V870.4c0 21.76-16.64 38.4-38.4 38.4zM268.8 115.2c-7.68 0-12.8 5.12-12.8 12.8v742.4c0 7.68 5.12 12.8 12.8 12.8h512c7.68 0 12.8-5.12 12.8-12.8V337.92L570.88 115.2H268.8z"
        fill="#231C1C" />
    <path d="M576 307.2c0 14.08 11.52 25.6 25.6 25.6h204.8L576 102.4v204.8z" fill="#008181" />
    <path
        d="M806.4 345.6H601.6c-21.76 0-38.4-16.64-38.4-38.4V102.4c0-5.12 2.56-10.24 7.68-11.52 5.12-2.56 10.24-1.28 14.08 2.56l230.4 230.4c3.84 3.84 5.12 8.96 2.56 14.08-1.28 5.12-6.4 7.68-11.52 7.68zM588.8 133.12V307.2c0 7.68 5.12 12.8 12.8 12.8h174.08L588.8 133.12zM332.8 435.2h371.2v25.6H332.8zM332.8 524.8h371.2v25.6H332.8z"
        fill="#231C1C" />
    <path d="M332.8 614.4h371.2v25.6H332.8z" fill="#231C1C" />
    <path d="M332.8 716.8h371.2v25.6H332.8z" fill="#231C1C" />
</svg>
"##;

pub const FOLDER_SVG_ICON: &str = r##"
<svg width="40px" height="40px" viewBox="0 0 1024 1024" class="icon" version="1.1"
        xmlns="http://www.w3.org/2000/svg" fill="#000000">
        <g id="SVGRepo_bgCarrier" stroke-width="0"></g>
        <g id="SVGRepo_tracerCarrier" stroke-linecap="round" stroke-linejoin="round"></g>
        <g id="SVGRepo_iconCarrier">
            <path
                d="M563.2 358.4c0 14.08-11.52 25.6-25.6 25.6H153.6c-14.08 0-25.6-11.52-25.6-25.6V166.4c0-14.08 11.52-25.6 25.6-25.6h230.4c14.08 0 25.6 11.52 25.6 25.6l153.6 192z"
                fill="#D3AC51"></path>
            <path
                d="M537.6 396.8H153.6c-21.76 0-38.4-16.64-38.4-38.4V166.4c0-21.76 16.64-38.4 38.4-38.4h230.4c19.2 0 35.84 14.08 38.4 33.28l153.6 192v5.12c0 21.76-16.64 38.4-38.4 38.4zM153.6 153.6c-7.68 0-12.8 5.12-12.8 12.8v192c0 7.68 5.12 12.8 12.8 12.8h384c5.12 0 10.24-3.84 12.8-8.96L396.8 171.52V166.4c0-7.68-5.12-12.8-12.8-12.8H153.6z"
                fill="#231C1C"></path>
            <path
                d="M921.6 768c0 14.08-11.52 25.6-25.6 25.6H153.6c-14.08 0-25.6-11.52-25.6-25.6V256c0-14.08 11.52-25.6 25.6-25.6h742.4c14.08 0 25.6 11.52 25.6 25.6v512z"
                fill="#FAC546"></path>
            <path
                d="M896 806.4H153.6c-21.76 0-38.4-16.64-38.4-38.4V256c0-21.76 16.64-38.4 38.4-38.4h742.4c21.76 0 38.4 16.64 38.4 38.4v512c0 21.76-16.64 38.4-38.4 38.4zM153.6 243.2c-7.68 0-12.8 5.12-12.8 12.8v512c0 7.68 5.12 12.8 12.8 12.8h742.4c7.68 0 12.8-5.12 12.8-12.8V256c0-7.68-5.12-12.8-12.8-12.8H153.6z"
                fill="#231C1C"></path>
        </g>
    </svg>
"##;

#[rustfmt::skip]
pub fn build_not_found_page() -> String {
    PAGE_TEMPLATE
        .replace("{title}", "Error Response")
        .replace("{content}", "<h1>404 Not Found</h1><p>Nothing matches the given URI</p>")
}
