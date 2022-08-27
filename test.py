def divide_file(filePath, chunkSize):

    readedDataSize = 0
    i = 0
    fileList = []

    # 対象ファイルを開く
    f = open(filePath, "rb")

    # ファイルを読み終わるまで繰り返す
    contentLength = os.path.getsize(filePath)
    while readedDataSize < contentLength:

        # 読み取り位置をシーク
        f.seek(readedDataSize)

        # 指定されたデータサイズだけ読み込む
        data = f.read(chunkSize)

        # 分割ファイルを保存
        saveFilePath = filePath + "." + str(i)
        with open(saveFilePath, 'wb') as saveFile:
            saveFile.write(data)

        # 読み込んだデータサイズの更新
        readedDataSize = readedDataSize + len(data)
        i = i + 1
        fileList.append(saveFilePath)

    return fileList