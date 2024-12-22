import { ModelTypes, scrapeTextFromUrls } from "./index.js"


const arr = [
    generateEmbeddings(["hi"], ModelTypes.AllMiniLML6V2),
    generateEmbeddings(["hi"], ModelTypes.BGESmallENV15),
    generateEmbeddings(["hi"], ModelTypes.MultilingualE5Large),
    generateEmbeddings(["hi"], ModelTypes.AllMiniLML12V2),
    generateEmbeddings(["hi"], ModelTypes.AllMiniLML6V2Q),
    generateEmbeddings(["hi"], ModelTypes.BGELargeENV15Q),
    generateEmbeddings(["hi"], ModelTypes.NomicEmbedTextV15),
    generateEmbeddings(["hi"], ModelTypes.BGEBaseENV15),
    generateEmbeddings(["hi"], ModelTypes.NomicEmbedTextV1),
    generateEmbeddings(["hi"], ModelTypes.BGEBaseENV15Q),
    generateEmbeddings(["hi"], ModelTypes.ParaphraseMLMiniLML12V2),
    generateEmbeddings(["hi"], ModelTypes.ParaphraseMLMpnetBaseV2),
    generateEmbeddings(["hi"], ModelTypes.MultilingualE5Small),
    generateEmbeddings(["hi"], ModelTypes.GTEBaseENV15),
    generateEmbeddings(["hi"], ModelTypes.GTELargeENV15),
    generateEmbeddings(["hi"], ModelTypes.MxbaiEmbedLargeV1),
    generateEmbeddings(["hi"], ModelTypes.MultilingualE5Base),
    generateEmbeddings(["hi"], ModelTypes.ParaphraseMLMiniLML12V2Q),
    generateEmbeddings(["hi"], ModelTypes.BGESmallENV15Q),
    generateEmbeddings(["hi"], ModelTypes.BGESmallZHV15),
    generateEmbeddings(["hi"], ModelTypes.MxbaiEmbedLargeV1Q),
    generateEmbeddings(["hi"], ModelTypes.NomicEmbedTextV15Q),
    generateEmbeddings(["hi"], ModelTypes.GTEBaseENV15Q),
    generateEmbeddings(["hi"], ModelTypes.AllMiniLML12V2Q),
    generateEmbeddings(["hi"], ModelTypes.GTELargeENV15Q),
    generateEmbeddings(["hi"], ModelTypes.BGELargeENV15)]


new Promise((reslove) => {

})