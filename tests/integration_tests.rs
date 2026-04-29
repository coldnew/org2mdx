use std::fs;

fn test_pair(stem: &str) {
    let org_path = format!("tests/org/{}.org", stem);
    let mdx_path = format!("tests/mdx/{}.mdx", stem);

    let org =
        fs::read_to_string(&org_path).unwrap_or_else(|e| panic!("Cannot read {}: {}", org_path, e));
    let expected =
        fs::read_to_string(&mdx_path).unwrap_or_else(|e| panic!("Cannot read {}: {}", mdx_path, e));

    let actual =
        org2mdx::convert(&org).unwrap_or_else(|e| panic!("Conversion failed for {}: {}", stem, e));

    if actual != expected {
        // Print diff-like output
        let actual_lines: Vec<&str> = actual.lines().collect();
        let expected_lines: Vec<&str> = expected.lines().collect();
        let max = actual_lines.len().max(expected_lines.len());
        let mut diff_shown = 0;
        for i in 0..max {
            let a = actual_lines.get(i).copied().unwrap_or("<missing>");
            let e = expected_lines.get(i).copied().unwrap_or("<missing>");
            if a != e {
                eprintln!("Line {} mismatch in {}:", i + 1, stem);
                eprintln!("  expected: {:?}", e);
                eprintln!("  actual:   {:?}", a);
                diff_shown += 1;
                if diff_shown >= 10 {
                    break;
                }
            }
        }
        panic!("Output mismatch for {}", stem);
    }
}

macro_rules! test_file {
    ($name:ident, $stem:expr) => {
        #[test]
        fn $name() {
            test_pair($stem);
        }
    };
}

test_file!(test_bootlin, "bootlin-的課程資源");
test_file!(test_android_gpio, "Android-Things-學習筆記-GPIO-輸出控制");
test_file!(test_android_rpi3, "Android-Things-學習筆記-RPI3-設定");
test_file!(test_android_intro, "Android-Things-學習筆記-前言");
test_file!(
    test_coscup,
    "COSCUP-2013-\"Org-Mode---emacs-下的瑞士軍刀\"-投影片"
);
test_file!(
    test_unknown_perls,
    "Unknown-perls-from-the-Clojure-standard-library-筆記"
);
test_file!(
    test_cpp_agent,
    "使用-C++-實現一個簡單的-Coding-Agent-從-curl-呼叫開始"
);
test_file!(
    test_clojure_javafx_webview,
    "使用-Clojure-和-JavaFX-Webview-打造桌面程式"
);
test_file!(test_clojure_currency, "使用-Clojure-擷取台灣銀行牌告匯率");
test_file!(
    test_clojurescript_node,
    "使用-ClojureScript-來寫-node.js-程式"
);
test_file!(test_codox, "使用-codox-與-CircleCI-建立-Clojure-專案的文檔");
test_file!(
    test_nix_emacs_ci,
    "使用-nix-emacs-ci-和-travis-ci-來測試-emacs-lisp-專案"
);
test_file!(test_opencode_kiro, "使用-opencode-搭配-kiro-進行開發");
test_file!(
    test_emacs_x11_ime,
    "修正-emacs-在-x11-下不能使用中文輸入法的問題"
);
test_file!(
    test_shell_dir,
    "切換-shell-到-emacs-目前正在編輯文件的資料夾"
);
test_file!(test_git_sites, "可以快速學習-git-的網站");
test_file!(test_jline, "在-clojure-下使用-JLine-2.x-實現互動式命令");
test_file!(
    test_generic_mode,
    "在-emacs-下使用-Generic-Mode-輕鬆建立新語言的語法上色"
);
test_file!(test_json_el, "在-emacs-下使用-json.el-來讀取-JSON-資料");
test_file!(test_mu4e, "在-emacs-下使用-mu4e-收發郵件");
test_file!(
    test_verify_url,
    "在-emacs-下使用-verify-url-檢查不存在的-URL"
);
test_file!(
    test_line_numbers,
    "在-emacs-下讓某些-major-mode-預設不顯示行號"
);
test_file!(test_easypg, "在-emacs-中使用-EasyPG-加密文章");
test_file!(test_wine, "在-linux-上使用-wine-執行-easybuilderpro");
test_file!(test_dynamic_modules, "淺談-emacs25-的-dynamic-modules-功能");
test_file!(test_clojure_javafx, "用-Clojure-寫-javafx-的-Hello-World");
test_file!(
    test_clojurescript_tty,
    "讓-Clojurescript-使用-node.js-的外部函式庫-以-tty.js-為例"
);
test_file!(
    test_emacs_space,
    "讓你的-emacs-自動在英文與中文之間加入空白"
);
test_file!(test_ext4, "關閉-ext4-的-journal-功能");
