import { useCallback, useState } from 'react';
import { useDropzone } from 'react-dropzone';
import { Upload, FileSpreadsheet, X, Download } from 'lucide-react';
import { Button } from '@/components/ui/Button';
import { cn } from '@/utils';
import { downloadTemplate } from '@/api/hotels';
import { downloadBlob } from '@/api/scrapeJobs';

interface ExcelUploaderProps {
  onFileSelect: (file: File) => void;
  selectedFile: File | null;
  onClear: () => void;
}

export function ExcelUploader({ onFileSelect, selectedFile, onClear }: ExcelUploaderProps) {
  const [isDownloading, setIsDownloading] = useState(false);

  const onDrop = useCallback(
    (acceptedFiles: File[]) => {
      if (acceptedFiles.length > 0) {
        onFileSelect(acceptedFiles[0]);
      }
    },
    [onFileSelect]
  );

  const { getRootProps, getInputProps, isDragActive } = useDropzone({
    onDrop,
    accept: {
      'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet': ['.xlsx'],
      'application/vnd.ms-excel': ['.xls'],
    },
    maxFiles: 1,
  });

  const handleDownloadTemplate = async () => {
    try {
      setIsDownloading(true);
      const blob = await downloadTemplate();
      downloadBlob(blob, 'hotel-import-template.xlsx');
    } catch (error) {
      console.error('Failed to download template:', error);
    } finally {
      setIsDownloading(false);
    }
  };

  if (selectedFile) {
    return (
      <div className="flex items-center gap-3 p-4 border rounded-lg bg-muted/30">
        <FileSpreadsheet className="h-8 w-8 text-green-600" />
        <div className="flex-1 min-w-0">
          <p className="font-medium truncate">{selectedFile.name}</p>
          <p className="text-sm text-muted-foreground">
            {(selectedFile.size / 1024).toFixed(1)} KB
          </p>
        </div>
        <Button variant="ghost" size="icon" onClick={onClear}>
          <X className="h-4 w-4" />
        </Button>
      </div>
    );
  }

  return (
    <div className="space-y-3">
      <div
        {...getRootProps()}
        className={cn(
          'border-2 border-dashed rounded-lg p-8 text-center cursor-pointer transition-colors',
          isDragActive
            ? 'border-primary bg-primary/5'
            : 'border-muted-foreground/25 hover:border-primary/50'
        )}
      >
        <input {...getInputProps()} />
        <Upload className="h-10 w-10 mx-auto mb-3 text-muted-foreground" />
        {isDragActive ? (
          <p className="text-primary font-medium">Drop the Excel file here...</p>
        ) : (
          <>
            <p className="font-medium">Drag & drop an Excel file here</p>
            <p className="text-sm text-muted-foreground mt-1">
              or click to select a file (.xlsx, .xls)
            </p>
          </>
        )}
      </div>
      <div className="flex justify-center">
        <Button
          variant="outline"
          size="sm"
          onClick={handleDownloadTemplate}
          disabled={isDownloading}
        >
          <Download className="h-4 w-4 mr-2" />
          {isDownloading ? 'Downloading...' : 'Download Template'}
        </Button>
      </div>
    </div>
  );
}
