export function RomSelector() {
  function handleUpload(file?: File) {
    console.log(file);
  }

  return (
    <div>
      <label>
        Upload ROM
        <input
          id="uploadedRom"
          type="file"
          accept=".ch8,.c8"
          multiple={false}
          onChange={({ target }) => handleUpload(target.files?.[0])}
        />
      </label>
    </div>
  );
}
