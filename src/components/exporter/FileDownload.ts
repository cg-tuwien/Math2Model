export function save(blob: Blob, filename: string) {
  const blobUrl = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = blobUrl;
  a.download = filename;
  a.click();
  URL.revokeObjectURL(blobUrl);
}
export function saveFileBinary(content: any, filename: string) {
  save(new Blob([content], { type: "application/octet-stream" }), filename);
}
export function saveFile(text: string, filename: string) {
  save(new Blob([text], { type: "text/plain" }), filename);
}
