# AWS LambdaでRust

## Cargo Lambda

### 1. [Cargo Lambdaのインストール](https://www.cargo-lambda.info/guide/installation.html)

#### 1-1. Scoopでインストール（Windows） <span class="col-red">NG</span>

- Scoopのインストール
    ```
    https://scoop.sh/
    > Set-ExecutionPolicy RemoteSigned -Scope CurrentUser # オプン: 最初のリモートスクリプトの実行に必要
    > irm get.scoop.sh | iex
    ```
    注意：セキュリティソフトによってトロイの木馬と見なされることがあります。

- Cargo Lambdaのインストール 
    ```
    > scoop bucket add cargo-lambda https://github.com/cargo-lambdascoop-cargo-lambda
    > scoop install cargo-lambda/cargo-lambda
    ```
    注意：やはりセキュリティソフトによってトロイの木馬と見なされるめ、インストールに失敗する。

#### 1-2. PyPlでインストール <span class="col-red">NG</span>

   ```
   > pip3 install cargo-lambda
   ```
   注意：セキュリティソフトによってトロイの木馬と見なされるため、イストールに失敗する。

#### 1-3. ソースコードからのビルド <span class="col-red">NG</span>

   注意：セキュリティソフトによってトロイの木馬と見なされるため、ビルド後、作成物の一部が削除されるためNG。

#### 1-4. Dockerでインストール

###### 1-4-1. play with docker (pwd)でインストール <span class="col-red">NG</span>

1. SSH接続

ローカルクライアント側で `ssh-keygen` を実行する。

1) プロキシのない環境ではSSH接続に成功する。
2) プロキシのある環境
      ```
      ssh -o 'ProxyCommand connect-proxy -H http://(proxyアドレス):8080 %h %p' ip172-18-0-50-cj44pjcsnmng00e1mlg0@direct.labs.play-with-docker.com
      ```
      注意：おそらく `ad.toa.co.jp` が port 22を開けていないため、接続に失敗する。

###### 1-4-2. WSLでDockerを使用する <span class="col-red">OK</span>

1. 普通にaptでインストール(docker engine)
```
    $ sudo apt update
    $ sudo apt install ca-certificates curl gnupg lsb-release
    $ sudo mkdir -p /etc/apt/keyrings
    $ curl --proxy http://(proxyアドレス):8080/ -fsSL https://download.docker.com/linux/ubuntu/gpg | sudo gpg --dearmor -o /etc/apt/keyrings/docker.gpg
    $ sudo chmod a+r /etc/apt/keyrings/docker.gpg
```
2. Dockerデーモンの起動 + プロキシ対策 (docker のために 別タブで起動しておく)

```
$ export http_proxy="http://(proxyアドレス).:8080/"
$ export https_proxy="http://(proxyアドレス).:8080/"
$ sudo -E dockerd
```
3. dockerをsudoなしで起動するためグループにユーザー追加
```
$ sudo gpasswd -a $USER docker
```

4. docker image の pull
```
$ docker pull ghcr.io/cargo-lambda/cargo-lambda
```
5. AWS CLI を含めたDockerfileを作成してbuild
- Dockerfile
```
FROM ghcr.io/cargo-lambda/cargo-lambda:latest

# AWS CLIのインストール
RUN apt-get update && \
    apt-get install -y unzip && \
    curl "https://awscli.amazonaws.com/awscli-exe-linux-x86_64.zip" -o "awscliv2.zip" && \
    unzip awscliv2.zip && \
    ./aws/install && \
    rm awscliv2.zip

# プロキシの設定
ENV http_proxy=http://(proxyアドレス):8080
ENV https_proxy=http://(proxyアドレス):8080
ENV no_proxy=localhost,127.0.0.1

CMD [ "bash" ]
```
- build コマンド
```
docker build --build-arg https_proxy=http://(proxyアドレス):8080  -t cargo_lambda .
```
6. コンテナ作成コマンド

```text
$ docker run \
 -v $PWD/rustlambda:/home/rustlambda \
 -e HTTP_PROXY=http://(proxyアドレス):8080 \
 -e HTTPS_PROXY=http://(proxyアドレス):8080 \
 --name cargo_lambda -p 8080:8080 \
 -it cargo_lambda /bin/bash
```
 -e HTTP_PROXY=http://(proxyアドレス):8080 \
 -e HTTPS_PROXY=http://(proxyアドレス):8080 \
 の設定は社外だと不要っぽい。。

7. aws configure で Access Keyなど設定
```
# aws configure
AWS Access Key ID [****************D3UM]: 
```
Access Keyは AWS -> IAM-> ユーザー-> セキュリティ認証情報-> アクセスキー で作成して取得

8. cargo lambdaで作成,build,deploy (deployはproxy環境では成功せず)
```
# cargo lambda new test
# cd test
# cargo lambda build
# cargo lambda deploy
```
cargo lambda だと gitignoreファイルは作成されるのに git repository は作成されない

9. zip形式で出力
```
# cargo lambda build --output-format zip
```
AWS Lambdaページ->コード->コードソース->アップロード元タブ
で .zipファイルを選択
target/lambda/<proj名>/bootstrap.zip をuploadで更新

## AWS-SDK for RUST
[公式](https://docs.aws.amazon.com/sdk-for-rust/latest/dg/getting-started.html)と
[DunamoDBへの登録](https://zenn.dev/nakamurus/articles/f575666a6a13ff268686)などを参照
1. Carrgo.tomlへのcratoの追加
versionは [crates.io](https://crates.io/)で調べる
```
[dependencies]
aws-config = "0.56.0"
aws-sdk-dynamodb = "0.29.0"
```
または cargo add "crato" でも追加される
```
# cargo add aws-config
```
2. build は --release をつけて行う
AWS Lambdaの unzipで 250MB以下の制約に引っ掛かる

# AWS S3 Event 
https://docs.aws.amazon.com/ja_jp/AmazonS3/latest/userguide/notification-content-structure.html
git