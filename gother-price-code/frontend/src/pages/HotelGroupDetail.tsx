import { useState } from 'react';
import { useParams, useNavigate, Link } from 'react-router-dom';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import {
  ArrowLeft,
  Download,
  Plus,
  Upload,
  Play,
  Loader2,
  History,
  Clock,
  Trash2,
} from 'lucide-react';
import { Button } from '@/components/ui/Button';
import { Input } from '@/components/ui/Input';
import { Label } from '@/components/ui/Label';
import { Card } from '@/components/ui/Card';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/Table';
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
  importMasterHotels,
  listGroupJobs,
} from '@/api/hotelGroups';
import { createScrapeJob, getScrapeResults, cancelScrapeJob, downloadBlob } from '@/api/scrapeJobs';
import {
  listScheduledScrapeConfigs,
  createScheduledScrapeConfig,
  deleteScheduledScrapeConfig,
} from '@/api/scheduledScrapeConfigs';
import apiClient from '@/api/client';
import { Badge } from '@/components/ui/Badge';
import { formatDate, formatRelativeTime, getStatusColor } from '@/utils/format';
import type { ScrapeJob, ScrapeMethod } from '@/types';

export function HotelGroupDetail() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const queryClient = useQueryClient();

  const [isAddHotelOpen, setIsAddHotelOpen] = useState(false);
  const [isImportOpen, setIsImportOpen] = useState(false);
  const [importFormat, setImportFormat] = useState<'plain' | 'master'>('plain');
  const [isScrapeFormOpen, setIsScrapeFormOpen] = useState(false);
  const [isProgressOpen, setIsProgressOpen] = useState(false);
  const [activeJobId, setActiveJobId] = useState<string | null>(null);

  const [hotelName, setHotelName] = useState('');
  const [hotelCity, setHotelCity] = useState('');
  const [hotelCountry, setHotelCountry] = useState('Thailand');
  const [importFile, setImportFile] = useState<File | null>(null);

  const [isScheduleOpen, setIsScheduleOpen] = useState(false);
  const [scheduleName, setScheduleName] = useState('');
  const [scheduleCron, setScheduleCron] = useState('0 6 * * 1');
  const [scheduleLookahead, setScheduleLookahead] = useState('7,30');
  const [scheduleMethod, setScheduleMethod] = useState<ScrapeMethod>('serpapi');

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

  // Fetch scheduled scrape configs (REQ-002 F-003/F-004)
  const { data: schedules } = useQuery({
    queryKey: ['scheduledScrapeConfigs', id],
    queryFn: () => listScheduledScrapeConfigs(id!),
    enabled: !!id,
  });

  // Poll the active job's full results (not just the aggregate progress
  // endpoint) so the progress dialog can show a per-hotel status log.
  const { data: activeResults } = useQuery({
    queryKey: ['scrapeResults', activeJobId],
    queryFn: () => getScrapeResults(activeJobId!),
    enabled: !!activeJobId && isProgressOpen,
    refetchInterval: (query) => {
      const status = query.state.data?.job.status;
      const isTerminal = status === 'completed' || status === 'failed' || status === 'cancelled';
      return isTerminal ? false : 5000;
    },
  });

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
    mutationFn: () =>
      importFormat === 'master'
        ? importMasterHotels(id!, importFile!)
        : importHotels(id!, importFile!),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['hotelGroup', id] });
      setIsImportOpen(false);
      setImportFile(null);
    },
  });

  // Create scheduled scrape config mutation
  const createScheduleMutation = useMutation({
    mutationFn: () =>
      createScheduledScrapeConfig({
        hotel_group_id: id!,
        name: scheduleName || undefined,
        cron_expression: scheduleCron,
        lookahead_days: scheduleLookahead
          .split(',')
          .map((s) => parseInt(s.trim(), 10))
          .filter((n) => !isNaN(n)),
        method: scheduleMethod,
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['scheduledScrapeConfigs', id] });
      setIsScheduleOpen(false);
      setScheduleName('');
      setScheduleCron('0 6 * * 1');
      setScheduleLookahead('7,30');
      setScheduleMethod('serpapi');
    },
  });

  // Delete scheduled scrape config mutation
  const deleteScheduleMutation = useMutation({
    mutationFn: (configId: string) => deleteScheduledScrapeConfig(configId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['scheduledScrapeConfigs', id] });
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

  const handleCreateSchedule = (e: React.FormEvent) => {
    e.preventDefault();
    if (scheduleCron.trim()) {
      createScheduleMutation.mutate();
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
    <div className="max-w-[1400px] mx-auto py-6 px-7">
      {/* Back link */}
      <Link
        to="/"
        className="inline-flex items-center gap-1.5 text-sm text-slate-500 hover:text-slate-900 mb-3.5"
      >
        <ArrowLeft className="h-3.5 w-3.5" />
        Back to Dashboard
      </Link>

      {/* Header */}
      <div className="flex items-start justify-between mb-6">
        <div>
          <h1 className="text-xl font-bold">🏨 {group.name}</h1>
          <p className="text-sm text-muted-foreground mt-1">
            {hotels.length} hotel{hotels.length !== 1 ? 's' : ''}
            {group.description && ` · ${group.description}`}
          </p>
        </div>
        <Button onClick={() => setIsScrapeFormOpen(true)} disabled={hotels.length === 0}>
          <Play className="h-4 w-4 mr-2" />
          New Price Search
        </Button>
      </div>

      {/* Actions */}
      <div className="flex items-center gap-2.5 mb-5">
        <Button variant="outline" onClick={() => setIsImportOpen(true)}>
          <Upload className="h-4 w-4 mr-2" />
          Import Excel
        </Button>
        <Button variant="outline" onClick={() => setIsAddHotelOpen(true)}>
          <Plus className="h-4 w-4 mr-2" />
          Add Hotel
        </Button>
        <Button
          variant="outline"
          className="ml-auto"
          onClick={async () => {
            const response = await apiClient.get('/export/price-history', {
              params: { hotel_group_id: id, format: 'csv' },
              responseType: 'blob',
            });
            downloadBlob(response.data, `${group.name.replace(/\s+/g, '-')}-price-history.csv`);
          }}
        >
          <Download className="h-4 w-4 mr-2" />
          Export Price History
        </Button>
      </div>

      {/* Hotels card */}
      <Card className="mb-5">
        <div className="px-5 py-3.5 border-b flex items-center justify-between">
          <h2 className="text-sm font-bold">Hotels ({hotels.length})</h2>
        </div>
        <HotelTable
          hotels={hotels}
          onRemove={(hotelId) => removeHotelMutation.mutate(hotelId)}
          isRemoving={removeHotelMutation.isPending}
        />
      </Card>

      {/* Scheduled Scrapes card */}
      <Card className="mb-5">
        <div className="px-5 py-3.5 border-b flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Clock className="h-4 w-4 text-muted-foreground" />
            <h2 className="text-sm font-bold">Scheduled Scrapes</h2>
          </div>
          <Button variant="outline" size="sm" onClick={() => setIsScheduleOpen(true)}>
            <Plus className="h-3.5 w-3.5 mr-1.5" />
            New Schedule
          </Button>
        </div>
        {schedules && schedules.length > 0 ? (
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Name</TableHead>
                <TableHead>Cron</TableHead>
                <TableHead>Method</TableHead>
                <TableHead>Status</TableHead>
                <TableHead>Next run</TableHead>
                <TableHead className="text-right">Actions</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {schedules.map((sched) => (
                <TableRow key={sched.id}>
                  <TableCell className="font-medium">{sched.name || '—'}</TableCell>
                  <TableCell className="font-mono text-xs text-muted-foreground">
                    {sched.cron_expression}
                  </TableCell>
                  <TableCell>
                    <Badge variant="info">{sched.method}</Badge>
                  </TableCell>
                  <TableCell>
                    <Badge className={sched.is_active ? 'bg-green-100 text-green-800' : 'bg-slate-100 text-slate-600'}>
                      {sched.is_active ? 'Active' : 'Paused'}
                    </Badge>
                  </TableCell>
                  <TableCell className="text-muted-foreground">
                    {sched.next_run_at ? formatRelativeTime(sched.next_run_at) : '—'}
                  </TableCell>
                  <TableCell className="text-right">
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => deleteScheduleMutation.mutate(sched.id)}
                      disabled={deleteScheduleMutation.isPending}
                    >
                      <Trash2 className="h-3.5 w-3.5 text-red-500" />
                    </Button>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        ) : (
          <div className="text-center py-8 text-muted-foreground text-sm">
            No scheduled scrapes yet. Create one to run this group automatically on a cron schedule.
          </div>
        )}
      </Card>

      {/* Recent Jobs card */}
      <Card>
        <div className="px-5 py-3.5 border-b flex items-center gap-2">
          <History className="h-4 w-4 text-muted-foreground" />
          <h2 className="text-sm font-bold">Recent Price Search Jobs</h2>
        </div>
        {jobs && jobs.length > 0 ? (
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Dates</TableHead>
                <TableHead>Guests</TableHead>
                <TableHead>Method</TableHead>
                <TableHead>Status</TableHead>
                <TableHead className="text-right">Created</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {jobs.map((job: ScrapeJob) => (
                <TableRow
                  key={job.id}
                  className="cursor-pointer"
                  onClick={() => navigate(`/reports/${job.id}`)}
                >
                  <TableCell className="font-medium">
                    {formatDate(job.checkin_date)} – {formatDate(job.checkout_date)}
                  </TableCell>
                  <TableCell className="text-muted-foreground">
                    {job.rooms} room{job.rooms !== 1 ? 's' : ''}, {job.adults} adult
                    {job.adults !== 1 ? 's' : ''}
                  </TableCell>
                  <TableCell>
                    <Badge variant="info">{job.method}</Badge>
                  </TableCell>
                  <TableCell>
                    <Badge className={getStatusColor(job.status)}>
                      {job.status.charAt(0).toUpperCase() + job.status.slice(1)}
                    </Badge>
                  </TableCell>
                  <TableCell className="text-right text-muted-foreground">
                    {formatRelativeTime(job.created_at)}
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        ) : (
          <div className="text-center py-12 text-muted-foreground">
            No past reports yet. Run a price scrape to create one.
          </div>
        )}
      </Card>

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
            <div className="py-4 space-y-4">
              <div>
                <Label className="mb-2 block">Format</Label>
                <div className="flex gap-4 text-sm">
                  <label className="flex items-center gap-1.5 cursor-pointer">
                    <input
                      type="radio"
                      name="importFormat"
                      checked={importFormat === 'plain'}
                      onChange={() => setImportFormat('plain')}
                    />
                    Simple — hotel_name, city, country
                  </label>
                  <label className="flex items-center gap-1.5 cursor-pointer">
                    <input
                      type="radio"
                      name="importFormat"
                      checked={importFormat === 'master'}
                      onChange={() => setImportFormat('master')}
                    />
                    Master hotel list — HID, Hotel-Name, UPDATE URL, SLUG, Supplier-or-Direct, Country
                  </label>
                </div>
              </div>
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

      {/* New Schedule Dialog */}
      <Dialog open={isScheduleOpen} onOpenChange={setIsScheduleOpen}>
        <DialogContent>
          <form onSubmit={handleCreateSchedule}>
            <DialogHeader>
              <DialogTitle>New Scheduled Scrape</DialogTitle>
              <DialogDescription>
                Runs this group's price search automatically on a recurring schedule.
              </DialogDescription>
            </DialogHeader>
            <div className="space-y-4 py-4">
              <div className="space-y-2">
                <Label htmlFor="schedule-name">Name</Label>
                <Input
                  id="schedule-name"
                  value={scheduleName}
                  onChange={(e) => setScheduleName(e.target.value)}
                  placeholder="e.g., Weekly Bangkok check"
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="schedule-cron">Cron expression *</Label>
                <Input
                  id="schedule-cron"
                  value={scheduleCron}
                  onChange={(e) => setScheduleCron(e.target.value)}
                  placeholder="0 6 * * 1"
                  className="font-mono"
                  required
                />
                <p className="text-xs text-muted-foreground">
                  Standard 5-field cron, e.g. <code>0 6 * * 1</code> = every Monday at 06:00.
                </p>
              </div>
              <div className="space-y-2">
                <Label htmlFor="schedule-lookahead">Lookahead days</Label>
                <Input
                  id="schedule-lookahead"
                  value={scheduleLookahead}
                  onChange={(e) => setScheduleLookahead(e.target.value)}
                  placeholder="7,30"
                />
                <p className="text-xs text-muted-foreground">
                  Comma-separated days ahead of the run date to check in (e.g. 7,30).
                </p>
              </div>
              <div className="space-y-2">
                <Label htmlFor="schedule-method">Method</Label>
                <select
                  id="schedule-method"
                  className="w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
                  value={scheduleMethod}
                  onChange={(e) => setScheduleMethod(e.target.value as ScrapeMethod)}
                >
                  <option value="serpapi">SerpAPI</option>
                  <option value="chatgpt">ChatGPT</option>
                  <option value="gemini">Gemini</option>
                  <option value="both">Both</option>
                </select>
              </div>
            </div>
            <DialogFooter>
              <Button type="button" variant="outline" onClick={() => setIsScheduleOpen(false)}>
                Cancel
              </Button>
              <Button type="submit" disabled={!scheduleCron.trim() || createScheduleMutation.isPending}>
                {createScheduleMutation.isPending ? 'Creating...' : 'Create Schedule'}
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
        results={activeResults ?? null}
        onCancel={() => cancelJobMutation.mutate()}
        onViewResults={handleViewResults}
        isLoading={cancelJobMutation.isPending}
      />
    </div>
  );
}
