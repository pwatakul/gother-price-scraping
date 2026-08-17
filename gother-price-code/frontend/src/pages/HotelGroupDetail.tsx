import { useMemo, useState } from 'react';
import { useParams, useNavigate, useSearchParams, Link } from 'react-router-dom';
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
  Pause,
  PlayCircle,
  Settings,
  BarChart3,
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
import {
  SearchConfigForm,
  describeSearchConfig,
  type SearchConfig,
} from '@/components/SearchConfigForm';
import { ProgressTracker } from '@/components/ProgressTracker';
import {
  getHotelGroup,
  addHotelToGroup,
  removeHotelFromGroup,
  importHotels,
  importMasterHotels,
  listGroupJobs,
  updateSearchConfig,
  runSavedSearch,
} from '@/api/hotelGroups';
import { getScrapeResults, cancelScrapeJob, downloadBlob } from '@/api/scrapeJobs';
import {
  listScheduledScrapeConfigs,
  createScheduledScrapeConfig,
  deleteScheduledScrapeConfig,
  updateScheduledScrapeConfig,
  runScheduledScrapeConfig,
} from '@/api/scheduledScrapeConfigs';
import apiClient from '@/api/client';
import { Pagination } from '@/components/Pagination';
import { Badge } from '@/components/ui/Badge';
import { formatDate, formatRelativeTime, getStatusColor } from '@/utils/format';
import type { ScrapeJob } from '@/types';

export function HotelGroupDetail() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const [searchParams, setSearchParams] = useSearchParams();
  const queryClient = useQueryClient();

  const [isAddHotelOpen, setIsAddHotelOpen] = useState(false);
  const [isImportOpen, setIsImportOpen] = useState(false);
  const [importFormat, setImportFormat] = useState<'plain' | 'master'>('plain');
  const [isSearchConfigOpen, setIsSearchConfigOpen] = useState(false);
  const [isProgressOpen, setIsProgressOpen] = useState(false);
  const [activeJobId, setActiveJobId] = useState<string | null>(null);

  const [hotelName, setHotelName] = useState('');
  const [hotelCity, setHotelCity] = useState('');
  const [hotelCountry, setHotelCountry] = useState('Thailand');
  const [importFile, setImportFile] = useState<File | null>(null);

  const [isScheduleOpen, setIsScheduleOpen] = useState(false);
  const [scheduleName, setScheduleName] = useState('');
  const [scheduleCron, setScheduleCron] = useState('0 6 * * 1');

  // Fetch hotel group details
  const { data, isLoading, error } = useQuery({
    queryKey: ['hotelGroup', id],
    queryFn: () => getHotelGroup(id!),
    enabled: !!id,
  });

  // Fetch past jobs — paginated server-side; page lives in the URL so it
  // survives a reload or a shared link, matching the hotels table.
  const jobPage = Number(searchParams.get('jobPage') ?? '0');
  const jobPageSize = Number(searchParams.get('jobPageSize') ?? '10');
  const { data: jobsPage } = useQuery({
    queryKey: ['groupJobs', id, jobPage, jobPageSize],
    queryFn: () => listGroupJobs(id!, jobPageSize, jobPage * jobPageSize),
    enabled: !!id,
  });
  const jobs = jobsPage?.jobs;
  const jobTotal = jobsPage?.total ?? 0;
  const jobTotalPages = Math.ceil(jobTotal / jobPageSize);
  // Clamp: jobs are only added, but page size can change under a deep link.
  const safeJobPage = Math.min(jobPage, Math.max(jobTotalPages - 1, 0));

  const updateJobPageParams = (updates: Record<string, number>, resetPage = true) => {
    const next = new URLSearchParams(searchParams);
    for (const [key, value] of Object.entries(updates)) next.set(key, String(value));
    if (resetPage) next.delete('jobPage');
    setSearchParams(next);
  };

  // Fetch scheduled scrape configs (REQ-002 F-003/F-004)
  const { data: schedules } = useQuery({
    queryKey: ['scheduledScrapeConfigs', id],
    queryFn: () => listScheduledScrapeConfigs(id!),
    enabled: !!id,
  });

  // The whole group arrives in one payload, so the hotels table paginates
  // client-side. Page lives in the URL so it survives a reload or a shared
  // link, matching HotelsList / HotelDetail.
  const hotelPage = Number(searchParams.get('hotelPage') ?? '0');
  const hotelPageSize = Number(searchParams.get('hotelPageSize') ?? '25');
  const allHotels = data?.hotels ?? [];
  const hotelTotalPages = Math.ceil(allHotels.length / hotelPageSize);
  // Clamp: removing hotels can leave the URL pointing past the last page.
  const safeHotelPage = Math.min(hotelPage, Math.max(hotelTotalPages - 1, 0));
  const pagedHotels = useMemo(
    () => allHotels.slice(safeHotelPage * hotelPageSize, (safeHotelPage + 1) * hotelPageSize),
    [allHotels, safeHotelPage, hotelPageSize]
  );

  // The group's saved search settings, shown in the summary line and used
  // to seed the settings dialog. Defaults mirror the DB defaults so the
  // dialog renders sensibly during the first load.
  const searchConfig: SearchConfig = {
    search_method: data?.group.search_method ?? 'serpapi',
    search_days_ahead: data?.group.search_days_ahead ?? [7],
    search_rooms: data?.group.search_rooms ?? 1,
    search_adults: data?.group.search_adults ?? 2,
  };

  const updateHotelPageParams = (updates: Record<string, number>, resetPage = true) => {
    const next = new URLSearchParams(searchParams);
    for (const [key, value] of Object.entries(updates)) next.set(key, String(value));
    if (resetPage) next.delete('hotelPage');
    setSearchParams(next);
  };

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
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['scheduledScrapeConfigs', id] });
      setIsScheduleOpen(false);
      setScheduleName('');
      setScheduleCron('0 6 * * 1');
    },
  });

  // Fire the standard grid now, without moving the next cron run
  const runScheduleMutation = useMutation({
    mutationFn: (configId: string) => runScheduledScrapeConfig(configId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['scheduledScrapeConfigs', id] });
      queryClient.invalidateQueries({ queryKey: ['groupJobs', id] });
    },
  });

  // Pause / resume without deleting the schedule
  const toggleScheduleMutation = useMutation({
    mutationFn: ({ configId, isActive }: { configId: string; isActive: boolean }) =>
      updateScheduledScrapeConfig(configId, { is_active: isActive }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['scheduledScrapeConfigs', id] });
    },
  });

  // Delete scheduled scrape config mutation
  const deleteScheduleMutation = useMutation({
    mutationFn: (configId: string) => deleteScheduledScrapeConfig(configId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['scheduledScrapeConfigs', id] });
    },
  });

  // Save the group's price-search settings (ADR-012)
  const saveSearchConfigMutation = useMutation({
    mutationFn: (config: SearchConfig) => updateSearchConfig(id!, config),
    onSuccess: () => {
      setIsSearchConfigOpen(false);
      queryClient.invalidateQueries({ queryKey: ['hotelGroup', id] });
    },
  });

  // Run the saved search now — check-in is derived server-side from the
  // stored days-ahead offset, so nothing is passed here.
  const runSearchMutation = useMutation({
    mutationFn: () => runSavedSearch(id!),
    onSuccess: (result) => {
      // The progress dialog tracks a single job, so only open it when the
      // run *was* a single job. A multi-window run is better read from the
      // jobs table than through one window's progress.
      if (result.job_ids.length === 1) {
        setActiveJobId(result.job_ids[0]);
        setIsProgressOpen(true);
      }
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
        <div className="flex flex-col items-end gap-1.5">
          <div className="flex items-center gap-2">
            <Button variant="outline" asChild>
              <Link to={`/groups/${id}/analytics`}>
                <BarChart3 className="h-4 w-4 mr-2" />
                View Analytics
              </Link>
            </Button>
            <Button variant="outline" onClick={() => setIsSearchConfigOpen(true)}>
              <Settings className="h-4 w-4 mr-2" />
              Search Settings
            </Button>
            <Button
              onClick={() => runSearchMutation.mutate()}
              disabled={hotels.length === 0 || runSearchMutation.isPending}
            >
              <Play className="h-4 w-4 mr-2" />
              {runSearchMutation.isPending ? 'Starting...' : 'Run Price Search'}
            </Button>
          </div>
          {/* What a run will actually do, without opening the dialog. */}
          <p className="text-xs text-muted-foreground">{describeSearchConfig(searchConfig)}</p>
        </div>
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
          hotels={pagedHotels}
          onRemove={(hotelId) => removeHotelMutation.mutate(hotelId)}
          isRemoving={removeHotelMutation.isPending}
        />
        {hotels.length > 0 && (
          <div className="px-5 pb-4">
            <Pagination
              page={safeHotelPage}
              totalPages={hotelTotalPages}
              totalItems={hotels.length}
              pageSize={hotelPageSize}
              onPageChange={(p) => updateHotelPageParams({ hotelPage: p }, false)}
              onPageSizeChange={(size) => updateHotelPageParams({ hotelPageSize: size })}
            />
          </div>
        )}
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
                <TableHead>Last run</TableHead>
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
                    <Badge variant="info">{searchConfig.search_method}</Badge>
                  </TableCell>
                  <TableCell>
                    <Badge className={sched.is_active ? 'bg-green-100 text-green-800' : 'bg-slate-100 text-slate-600'}>
                      {sched.is_active ? 'Active' : 'Paused'}
                    </Badge>
                  </TableCell>
                  <TableCell className="text-muted-foreground">
                    {sched.last_run_at ? formatRelativeTime(sched.last_run_at) : 'Never'}
                  </TableCell>
                  <TableCell className="text-muted-foreground">
                    {sched.next_run_at ? formatRelativeTime(sched.next_run_at) : '—'}
                  </TableCell>
                  <TableCell className="text-right">
                    <Button
                      variant="ghost"
                      size="sm"
                      title="Run the standard grid now (does not change the next scheduled run)"
                      onClick={() => runScheduleMutation.mutate(sched.id)}
                      disabled={runScheduleMutation.isPending}
                    >
                      <Play className="h-3.5 w-3.5 text-green-600" />
                    </Button>
                    <Button
                      variant="ghost"
                      size="sm"
                      title={sched.is_active ? 'Pause this schedule' : 'Resume this schedule'}
                      onClick={() =>
                        toggleScheduleMutation.mutate({
                          configId: sched.id,
                          isActive: !sched.is_active,
                        })
                      }
                      disabled={toggleScheduleMutation.isPending}
                    >
                      {sched.is_active ? (
                        <Pause className="h-3.5 w-3.5 text-amber-600" />
                      ) : (
                        <PlayCircle className="h-3.5 w-3.5 text-slate-500" />
                      )}
                    </Button>
                    <Button
                      variant="ghost"
                      size="sm"
                      title="Delete this schedule"
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
        {jobTotal > 0 && (
          <div className="px-5 pb-4">
            <Pagination
              page={safeJobPage}
              totalPages={jobTotalPages}
              totalItems={jobTotal}
              pageSize={jobPageSize}
              onPageChange={(p) => updateJobPageParams({ jobPage: p }, false)}
              onPageSizeChange={(size) => updateJobPageParams({ jobPageSize: size })}
              pageSizeOptions={[10, 25, 50]}
            />
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
        <DialogContent className="sm:max-w-lg">
          <form onSubmit={handleImport} className="min-w-0">
            <DialogHeader>
              <DialogTitle>Import Hotels</DialogTitle>
              <DialogDescription>
                Upload an Excel or CSV file to import multiple hotels
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
                <Label>What each run collects</Label>
                <div className="rounded-md border border-input bg-muted/40 px-3 py-2.5 text-xs text-muted-foreground space-y-1">
                  <p>
                    <span className="font-medium text-foreground">Booking windows:</span> +1, +3,
                    +7, +14, +30 days ahead
                  </p>
                  <p>
                    <span className="font-medium text-foreground">Devices:</span> desktop and mobile
                  </p>
                  <p>
                    <span className="font-medium text-foreground">Stay:</span> 1 night, 1 room, 2
                    adults
                  </p>
                  <p className="pt-1">
                    Fixed standard — 10 price searches per run, so every hotel stays comparable.
                  </p>
                </div>
              </div>
              <div className="space-y-2">
                <Label>Method</Label>
                <p className="text-xs text-muted-foreground">
                  Uses this group's configured method (
                  <span className="font-medium text-foreground">
                    {describeSearchConfig(searchConfig).split(' · ')[0]}
                  </span>
                  ). Change it in Search Settings so manual and scheduled runs stay in step.
                </p>
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

      {/* Saved price-search settings */}
      <SearchConfigForm
        open={isSearchConfigOpen}
        onOpenChange={setIsSearchConfigOpen}
        config={searchConfig}
        onSubmit={(config) => saveSearchConfigMutation.mutate(config)}
        isLoading={saveSearchConfigMutation.isPending}
        hotelCount={hotels.length}
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
