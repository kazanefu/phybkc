# design

## 概要

キーボード配列のプロファイル設定/管理とAutoHotkeyのようなカスタムショートカットキーの設定。

キーの名前ではなく物理キーの位置に基づいて設定できる。

例えば `Aキーが押されたとき`みたいなキーの名前による設定だとプロファイルによって押す場所が変わってしまうので代わりにキーの場所に基づいて設定できるようにする。

キー配列の設定はGUIで行い、それに基づいてプロファイル設定ファイルを作成する。

プロファイル設定ファイルはJSON形式で保存するつもり(もしかしたらTOMLにするかも)。

プロファイル設定ファイルに基づいて常駐のデーモンがキー入力を監視。また、タスクトレイに現在有効のプロファイルを表示し、さらにプロファイル切り替えもできるようにしたい。

ショートカットキー設定は専用スクリプトを用意しAutoHotkeyのようなスクリプトエンジンで実行する(これもデーモンで管理)。

ショートカットキーのスクリプトはGUIですべてのプロファイルに適用するもの/特定のプロファイルに適用するものを分けてインポートし、プロファイル設定ファイルに使うショートカットのリストをパスで保存する。(いずれはスクリプトもGUIで生成できるようにしたいが今は手書きで)

また、GUIで設定したプロファイルはエクスポート/インポートできるようにする。エクスポートするとき同じフォルダにスクリプトも複製する。

スクリプトは押されるキーの条件と実行するアクションのリストで構成される。(AutoHotkeyのスクリプトを参考に、ただしキーの指定は物理的な位置に基づくどのプロファイルでも変わらないものとキーの名前で指定するプロファイルによって位置が変化するものがある。)

プロファイルと全プロファイル共通のスクリプトリストをTOMLファイルに

```toml
[profiles]
profileA = "C:/User/phybkc/profiles/profileA.json"
profileB = "D:profiles/profileB.json"
qwerty = "C:/User/phybkc/profiles/qwerty.json"

[default_profile]
default = profileA

[global_scripts]
scriptA = "C:/User/phybkc/scripts/scriptA.phybkc"
hogehoge = "D:scripts/hogehoge.phybkc"
```

みたいな感じにして
プロファイル設定ファイルは

```json
{
    "name": "profileA",
    "keyboard": "JIS",
    "scripts": [
        "C:/User/phybkc/scripts/scriptA.phybkc"
    ],
    "keys": {
        "0xhogehoge": "A",
        "0xfugafuga": "LeftShift",
        ..
    }

}
```

みたいな感じにする

スクリプトの構文は

```phybkc
// コメントはダブルスラッシュ
// セミコロンは必須

// Runで使うCLIを指定
// Command Prompt/WSL/PowerShellなどいろいろ設定可能
// なくてもOSのデフォルトので起動可能
CLI = PowerShell;

// 物理キーでの指定はダブルクォーテーション, プロファイル依存のキーの名前での指定はCode_hoge
// +で同時に押されている必要のあるものを付け足していける
// Code_A + Code_BならばA -> Bの順で入力を受け付け, AとBが両方とも押されている状態に呼ばれる
// つまりCode_A + Code_BとCode_B + Code_Aで別々の定義が可能
"0xhoge" + "0xfuga" + Code_A{
    // Runは指定したCLIでコマンドを実行する
    // Executeは指定したアプリケーションで実行する(指定は絶対パス)
    Run: "mkdir test";
    Execute: "C:/Program Files/APP/APP.exe";
}

"0xfoo" + Code_LeftShift{
    // wait_inputは指定したキーが押されるまで待機する
    // 待機中に他のキーが押された場合は待機を中断する
    if wait_input("0xhogehoge" + "0xfoofoo"){
        Run: "wsl ~";
    } elif wait_input("0xhogehoge" + "0xfoofoo"){
        Run: "wsl ~";
    } else {
        Run: "wsl ~";
    }

    // wait_input_timeは指定したキーが指定した時間内に押されたらtrueを返す
    // 時間はミリ秒
    if wait_input_time("0xhogehoge" + "0xfoofoo", 10){
        Execute: "C:/Program Files/APP/APP.exe";
    }

    // now_inputは待たずに現在の入力状態をチェックする
    if now_input("0xhogehoge" + "0xfoofoo"){
        // loopは指定した回数だけ指定した処理を繰り返す
        loop 10{
            Run: "ls";
        }
    }

    // wait_releasedは指定したキーが離されるまで待機する
    // 待機中に他のキーが押された場合は待機を中断する
    if wait_released("0xhogehoge" + "0xfoofoo"){
        Run: "wsl ~";
    }

    // wait_released_timeは指定したキーが指定した時間内に離されたらtrueを返す
    // 時間はミリ秒
    if wait_released_time("0xhogehoge" + "0xfoofoo", 10){
        Execute: "C:/Program Files/APP/APP.exe";
    }
}
// マクロの定義
macro HOGE{
    Run: "cd ~";
    // waitは指定した時間だけ待機する
    // ミリ秒単位
    wait(1000);
    Run: "ls";
}

// マクロの呼び出し, 最後に!をつける
Code_Tab + Code_H{
    HOGE!;
}

```

みたいな感じ
**スクリプトに仕様変更**: `"0x2E"`のように表記するのではなく`#0x2E`のようにする

## crates

**daemon**:

- タスクトレイ常駐
- OS依存(まずはWindows11)
- 有効になっているプロファイル設定に基づいて入力変換
- 低レベルキーフック(とりあえず`SetWindowsHookEx` (WH_KEYBOARD_LL) を使用する)
- スクリプトの実行
- 右クリックメニュー: プロファイル切り替え, GUI起動, stop, exit, restart

**gui**:

- キーボードのレイアウトを表示
- プロファイル設定ファイルの生成/インポート/エクスポートをする
- プロファイルの削除

**profile**:

- プロファイル設定ファイルの読み込み/書き込みなどの共有するライブラリ

**cli**:

- GUIを完成させるまでのデバッグ用でGUIですることを大体コマンドラインでできるようにする

## 物理キーの識別方法

ScanCode

## GUIとデーモンの通信方法

基本的に通信はしない
プロファイル設定などを更新するときには一度デーモンを終了し、Activateボタンを押すことで再起動する

## キーボードレイアウトの考慮

プロファイル編集画面でレイアウト選択するようにする
ScanCode の差異もここで判断する (この項目をプロファイル設定ファイルにも含めるように変更した)

## スクリプトの同時実行

GUIでプロファイルを編集するときにスクリプトを追加するときに順番も指定するようにして順番の小さい方を優先するようにする

## 例外処理

Runで指定したコマンドが失敗した場合は、音だけ出して特に何もしない
新たにTryRunとTryExecuteを定義する。
TryRun/TryExecuteは失敗した時にFailRunかFailExecuteを実行する。
TryRun: "mkdir test":FailExecute: "C:/Program Files/APP/APP.exe";
TryExecute: "C:/Program Files/APP/APP.exe":FailRun: "echo error";

## 入力エミュレート

SendInputで入力エミュレートする。
スクリプトでの構文は

```phybkc
Send: Code_A;
Send: "0xhoge" + Code_B;
Send: String("Hello");
Send: String("Hello") + Code_Enter;
```

## スクリプト実行について

必ず並列処理を使ってスクリプト内で重い処理があっても他に影響が出ないようにする
