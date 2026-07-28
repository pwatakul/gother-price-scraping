import { useState, useEffect } from 'react';
import { useParams, useNavigate, Link } from 'react-router-dom';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import {
  ArrowLeft,
  Plus,
  Upload,
  Play,
  Loader2,
  History,
} from 'lucide-react';
import { Button } from '@/components/ui/Button';
import { Input } from '@/components/ui/Input';
import { Label } from '@/components/ui/Label';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/Tabs';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/Dialog';
import { HotelTable } from '@/components/HotelTable';
import { ExcelUploader } from '@/components/ExcelUploader';
import { ScrapeJobForm, ScrapeJobFormData } from '@/components/ScrapeJobForm';
import { ProgressTracker } from '@/components/ProgressTracker';
import {
  getHotelGroup,
  addHotelToGroup,
  removeHotelFromGroup,
  importHotels,
  listGroupJobs,
} from '@/api/hotelGroups';
import { createScrapeJob, getScrapeJob, cancelScrapeJob } from '@/api/scrapeJobs';
import { Badge } from '@/components/ui/Badge';
import { formatDate, formatRelativeTime, getStatusColor } from '@/utils/format';
import type { ScrapeJob, ScrapeJobWithProgress } from '@/types';

export function HotelGroupDetail() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const queryClient = useQueryClient();

  const [isAddHotelOpen, setIsAddHotelOpen] = useState(false);
  const [isImportOpen, setIsImportOpen] = useState(false);
  const [isScrapeFormOpen, setIsScrapeFormOpen] = useState(false);
  const [isProgressOpen, setIsProgressOpen] = useState(false);
  const [activeJobId, setActiveJobId] = useState<string | null>(null);
  const [activeJob, setActiveJob] = useState<ScrapeJobWithProgress | null>(null);

  const [hotelName, setHotelName] = useState('');
  const [hotelCity, setHotelCity] = useState('');
  const [hotelCountry, setHotelCountry] = useState('Thailand');
  const [importFile, setImportFile] = useState<File | null>(null);

  // Fetch hotel group details
  const { data, isLoading, error } = useQuery({
    queryKey: ['hotelGroup', id],
    queryFn: () => getHotelGroup(id!),
    enabled: !!id,
  });

  // Fetch past jobs
  const { data: jobs } = useQuery({
    queryKey: ['groupJobs', id],
    queryFn: () => listGroupJobs(id!, 10),
    enabled: !!id,
  });

  // Poll active job
  useEffect(() => {
    if (!activeJobId || !isProgressOpen) return;

    const pollJob = async () => {
      try {
        const job = await getScrapeJob(activeJobId);
        setActiveJob(job);

        if (job.status === 'completed' || job.status === 'failed' || job.status === 'cancelled') {
          // Stop polling
          return;
        }
      } catch (error) {
        console.error('Failed to poll job:', error);
      }
    };

    // Initial fetch
    pollJob();

    // Poll every 5 seconds
    const interval = setInterval(pollJob, 5000);
    return () => clearInterval(interval);
  }, [activeJobId, isProgressOpen]);

  // Add hotel mutation
  const addHotelMutation = useMutation({
    mutationFn: () =>
      addHotelToGroup(id!, {
        name: hotelName,
        city: hotelCity,
        country: hotelCountry,
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['hotelGroup', id] });
      setIsAddHotelOpen(false);
      setHotelName('');
      setHotelCity('');
      setHotelCountry('Thailand');
    },
  });

  // Remove hotel mutation
  const removeHotelMutation = useMutation({
    mutationFn: (hotelId: string) => removeHotelFromGroup(id!, hotelId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['hotelGroup', id] });
    },
  });

  // Import hotels mutation
  const importMutation = useMutation({
    mutationFn: () => importHotels(id!, importFile!),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['hotelGroup', id] });
      setIsImportOpen(false);
      setImportFile(null);
    },
  });

  // Create scrape job mutation
  const createJobMutation = useMutation({
    mutationFn: (data: ScrapeJobFormData) =>
      createScrapeJob({
        hotel_group_id: id!,
        ...data,
      }),
    onSuccess: (job) => {
      setIsScrapeFormOpen(false);
      setActiveJobId(job.id);
      setIsProgressOpen(true);
      queryClient.invalidateQueries({ queryKey: ['groupJobs', id] });
    },
  });

  // Cancel job mutation
  const cancelJobMutation = useMutation({
    mutationFn: () => cancelScrapeJob(activeJobId!),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['groupJobs', id] });
    },
  });

  const handleAddHotel = (e: React.FormEvent) => {
    e.preventDefault();
    addHotelMutation.mutate();
  };

  const handleImport = (e: React.FormEvent) => {
    e.preventDefault();
    if (importFile) {
      importMutation.mutate();
    }
  };

  const handleViewResults = () => {
    if (activeJobId) {
      setIsProgressOpen(false);
      navigate(`/reports/${activeJobId}`);
    }
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-12">
        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    );
  }

  if (error || !data) {
    return (
      <div className="container mx-auto py-8 px-4">
        <div className="text-center py-12">
          <h2 className="text-xl font-semibold mb-2">Hotel group not found</h2>
          <Button asChild>
            <Link to="/">Go back to Dashboard</Link>
          </Button>
        </div>
      </div>
    );
  }

  const { group, hotels } = data;

  return (
    <div className="container mx-auto py-8 px-4">
      {/* Header */}
      <div className="mb-6">
        <Button variant="ghost" asChild className="mb-4">
          <Link to="/">
            <ArrowLeft className="h-4 w-4 mr-2" />
            Back to Dashboard
          </Link>
        </Button>
        <div className="flex items-start justify-between">
          <div>
            <h1 className="text-3xl font-bold">{group.name}</h1>
            {group.description && (
              <p className="text-muted-foreground mt-1">{group.description}</p>
            )}
            <p className="text-sm text-muted-foreground mt-2">
              {hotels.length} hotel{hotels.length !== 1 ? 's' : ''}
            </p>
          </div>
          <Button
            size="lg"
            onClick={() => setIsScrapeFormOpen(true)}
            disabled={hotels.length === 0}
          >
            <Play className="h-4 w-4 mr-2" />
            Run Price Scrape
          </Button>
        </div>
      </div>

      {/* Tabs */}
      <Tabs defaultValue="hotels">
        <TabsList>
          <TabsTrigger value="hotels">Hotels</TabsTrigger>
          <TabsTrigger value="history">
            <History className="h-4 w-4 mr-2" />
            Past Reports
          </TabsTrigger>
        </TabsList>

        {/* Hotels Tab */}
        <TabsContent value="hotels" className="mt-6">
          <div className="flex items-center gap-3 mb-4">
            <Button variant="outline" onClick={() => setIsAddHotelOpen(true)}>
              <Plus className="h-4 w-4 mr-2" />
              Add Hotel
            </Button>
            <Button variant="outline" onClick={() => setIsImportOpen(true)}>
              <Upload className="h-4 w-4 mr-2" />
              Import Excel
            </Button>
          </div>
          <HotelTable
            hotels={hotels}
            onRemove={(hotelId) => removeHotelMutation.mutate(hotelId)}
            isRemoving={removeHotelMutation.isPending}
          />
        </TabsContent>

        {/* History Tab */}
        <TabsContent value="history" className="mt-6">
          {jobs && jobs.length > 0 ? (
            <div className="space-y-3">
              {jobs.map((job: ScrapeJob) => (
                <div
                  key={job.id}
                  className="flex items-center justify-between p-4 border rounded-lg hover:bg-muted/30 cursor-pointer"
                  onClick={() => navigate(`/reports/${job.id}`)}
                >
                  <div>
                    <div className="font-medium">
                      {formatDate(job.checkin_date)} - {formatDate(job.checkout_date)}
                    </div>
                    <div className="text-sm text-muted-foreground">
                      {job.rooms} room{job.rooms !== 1 ? 's' : ''}, {job.adults} adult
                      {job.adults !== 1 ? 's' : ''} • {formatRelativeTime(job.created_at)}
                    </div>
                  </div>
                  <Badge className={getStatusColor(job.status)}>
                    {job.status.charAt(0).toUpperCase() + job.status.slice(1)}
                  </Badge>
                </div>
              ))}
            </div>
          ) : (
            <div className="text-center py-12 text-muted-foreground">
              No past reports yet. Run a price scrape to create one.
            </div>
          )}
        </TabsContent>
      </Tabs>

      {/* Add Hotel Dialog */}
      <Dialog open={isAddHotelOpen} onOpenChange={setIsAddHotelOpen}>
        <DialogContent>
          <form onSubmit={handleAddHotel}>
            <DialogHeader>
              <DialogTitle>Add Hotel</DialogTitle>
              <DialogDescription>
                Add a single hotel to this group
              </DialogDescription>
            </DialogHeader>
            <div className="space-y-4 py-4">
              <div className="space-y-2">
                <Label htmlFor="hotel-name">Hotel Name *</Label>
                <Input
                  id="hotel-name"
                  value={hotelName}
                  onChange={(e) => setHotelName(e.target.value)}
                  placeholder="e.g., Dusit Thani Bangkok"
                  required
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="hotel-city">City *</Label>
                <Input
                  id="hotel-city"
                  value={hotelCity}
                  onChange={(e) => setHotelCity(e.target.value)}
                  placeholder="e.g., Bangkok"
                  required
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="hotel-country">Country</Label>
                <Input
                  id="hotel-country"
                  value={hotelCountry}
                  onChange={(e) => setHotelCountry(e.target.value)}
                  placeholder="e.g., Thailand"
                />
              </div>
            </div>
            <DialogFooter>
              <Button
                type="button"
                variant="outline"
                onClick={() => setIsAddHotelOpen(false)}
              >
                Cancel
              </Button>
              <Button type="submit" disabled={addHotelMutation.isPending}>
                {addHotelMutation.isPending ? 'Adding...' : 'Add Hotel'}
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>

      {/* Import Dialog */}
      <Dialog open={isImportOpen} onOpenChange={setIsImportOpen}>
        <DialogContent>
          <form onSubmit={handleImport}>
            <DialogHeader>
              <DialogTitle>Import Hotels</DialogTitle>
              <DialogDescription>
                Upload an Excel file to import multiple hotels
              </DialogDescription>
            </DialogHeader>
            <div className="py-4">
              <ExcelUploader
                onFileSelect={setImportFile}
                selectedFile={importFile}
                onClear={() => setImportFile(null)}
              />
            </div>
            <DialogFooter>
              <Button
                type="button"
                variant="outline"
                onClick={() => {
                  setIsImportOpen(false);
                  setImportFile(null);
                }}
              >
                Cancel
              </Button>
              <Button
                type="submit"
                disabled={!importFile || importMutation.isPending}
              >
                {importMutation.isPending ? 'Importing...' : 'Import Hotels'}
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>

      {/* Scrape Job Form */}
      <ScrapeJobForm
        open={isScrapeFormOpen}
        onOpenChange={setIsScrapeFormOpen}
        onSubmit={(data) => createJobMutation.mutate(data)}
        isLoading={createJobMutation.isPending}
      />

      {/* Progress Tracker */}
      <ProgressTracker
        open={isProgressOpen}
        onOpenChange={setIsProgressOpen}
        job={activeJob}
        onCancel={() => cancelJobMutation.mutate()}
        onViewResults={handleViewResults}
        isLoading={cancelJobMutation.isPending}
      />
    </div>
  );
}
